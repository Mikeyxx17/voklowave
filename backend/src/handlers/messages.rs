//! 消息硬删除 + 模糊搜索：DELETE /api/messages/{id}、GET /api/messages/search。
//!
//! 采用硬删除 + 墓碑表方案：消息从数据库彻底移除，同时写入 deleted_messages
//! 表（保留 1 小时），并通过控制事件广播通知所有在线客户端。
//! 重连客户端回放历史时，会先接收墓碑列表确保不留幽灵消息。
//!
//! 搜索使用 PostgreSQL pg_trgm 三元组索引加速 ILIKE 模糊匹配，
//! 支持按频道过滤和分页。

use crate::middleware::auth::AuthUser;
use crate::state::{AppState, ControlEvent};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;

/// 删除指定 ID 的消息。
/// - 仅消息发送者（按 username 匹配）可删除
/// - 物理删除消息记录
/// - 写入墓碑表（供重连客户端同步删除）
/// - 通过控制通道广播删除事件给所有在线客户端
pub async fn delete_message(
    State(state): State<AppState>,
    user: AuthUser,
    Path(message_id): Path<i32>,
) -> impl IntoResponse {
    // ── 查询消息，确认存在且属于当前用户 ──
    let msg = sqlx::query!(
        "SELECT id, channel, username FROM messages WHERE id = $1",
        message_id
    )
    .fetch_optional(&state.db)
    .await;

    let msg = match msg {
        Ok(Some(m)) => m,
        Ok(None) => return (StatusCode::NOT_FOUND, "消息不存在").into_response(),
        Err(e) => {
            println!("[消息删除] 查询消息失败: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // ── 权限校验：只能删除自己发送的消息 ──
    if msg.username != user.username {
        return (StatusCode::FORBIDDEN, "只能删除自己发送的消息").into_response();
    }

    // ── 开启事务：物理删除 + 墓碑写入 ──
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            println!("[消息删除] 开启事务失败: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // 物理删除消息
    let del_result = sqlx::query!("DELETE FROM messages WHERE id = $1", message_id)
        .execute(&mut *tx)
        .await;

    // 写入墓碑表（用于重连客户端同步）
    let tomb_result = sqlx::query!(
        "INSERT INTO deleted_messages (id, channel) VALUES ($1, $2)",
        message_id,
        msg.channel
    )
    .execute(&mut *tx)
    .await;

    match (del_result, tomb_result) {
        (Ok(_), Ok(_)) => {
            if let Err(e) = tx.commit().await {
                println!("[消息删除] 提交事务失败: {:?}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }

            // ── 通过控制通道广播删除事件 ──
            let control_tx = state.get_or_create_control_channel(&msg.channel);
            let _ = control_tx.send(ControlEvent::MessageDeleted {
                message_id,
            });

            println!(
                "[消息删除] 用户 {} 删除了频道 {} 中的消息 id={}",
                user.username, msg.channel, message_id
            );

            StatusCode::NO_CONTENT.into_response()
        }
        (err_del, err_tomb) => {
            println!(
                "[消息删除] 操作失败，del: {:?}, tomb: {:?}",
                err_del, err_tomb
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 消息搜索
// ═══════════════════════════════════════════════════════════════════════════

/// 搜索请求查询参数。
#[derive(Deserialize)]
pub struct SearchQuery {
    /// 搜索关键词（必填）
    pub q: String,
    /// 限制在指定频道内搜索（可选）
    pub channel: Option<String>,
    /// 偏移量，用于分页（默认 0）
    #[serde(default)]
    pub offset: i64,
    /// 每页条数（默认 20，最大 50）
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    20
}

/// 搜索结果的数据库行映射。
/// 使用 query_as 而非 query! 避免 if/else 分支类型不兼容问题。
#[derive(sqlx::FromRow)]
struct SearchRow {
    id: i32,
    channel: String,
    username: String,
    content: String,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 模糊搜索历史消息。
/// - 使用 ILIKE 模糊匹配消息内容（pg_trgm 索引自动加速）
/// - 支持按频道过滤（访客只能搜索 general 频道的消息）
/// - 搜索结果按时间倒序排列
/// - 访客也能使用搜索功能
pub async fn search_messages(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    // ── 参数校验 ──
    let q = query.q.trim();
    if q.is_empty() {
        return (StatusCode::BAD_REQUEST, "搜索关键词不能为空").into_response();
    }
    if q.len() > 100 {
        return (StatusCode::BAD_REQUEST, "搜索关键词不能超过 100 个字符").into_response();
    }

    let limit = query.limit.min(50).max(1);
    let offset = query.offset.max(0);

    // ── 构建 LIKE 模式（前后加 % 实现包含匹配） ──
    let pattern = format!("%{}%", q);

    // ── 确定查询的 SQL 和参数（统一类型，避免 if/else 分支不兼容） ──
    // 根据是否指定频道 + 是否为访客来决定过滤条件
    let channel_filter: Option<&str> = if user.is_guest {
        // 访客：限定 general 频道（channel 参数存在时也会被校验）
        if let Some(ref c) = query.channel {
            if c != "general" {
                return (StatusCode::FORBIDDEN, "访客只能搜索 general 频道").into_response();
            }
        }
        Some("general")
    } else {
        // 注册用户：可选频道过滤
        query.channel.as_deref()
    };

    // ── 执行数据查询 ──
    let rows = if let Some(ch) = channel_filter {
        sqlx::query_as::<_, SearchRow>(
            "SELECT id, channel, username, content, created_at \
             FROM messages \
             WHERE channel = $1 AND content ILIKE $2 \
             ORDER BY created_at DESC \
             LIMIT $3 OFFSET $4",
        )
        .bind(ch)
        .bind(&pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, SearchRow>(
            "SELECT id, channel, username, content, created_at \
             FROM messages \
             WHERE content ILIKE $1 \
             ORDER BY created_at DESC \
             LIMIT $2 OFFSET $3",
        )
        .bind(&pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
    };

    // ── 统计匹配总条数（用于分页） ──
    let total: i64 = if let Some(ch) = channel_filter {
        sqlx::query_scalar::<_, Option<i64>>(
            "SELECT COUNT(*) FROM messages WHERE channel = $1 AND content ILIKE $2",
        )
        .bind(ch)
        .bind(&pattern)
        .fetch_one(&state.db)
        .await
        .ok()
        .flatten()
        .unwrap_or(0)
    } else {
        sqlx::query_scalar::<_, Option<i64>>(
            "SELECT COUNT(*) FROM messages WHERE content ILIKE $1",
        )
        .bind(&pattern)
        .fetch_one(&state.db)
        .await
        .ok()
        .flatten()
        .unwrap_or(0)
    };

    // ── 构建响应 ──
    match rows {
        Ok(messages) => {
            let results: Vec<serde_json::Value> = messages
                .into_iter()
                .map(|row| {
                    serde_json::json!({
                        "id": row.id,
                        "channel": row.channel,
                        "username": row.username,
                        "content": row.content,
                        "created_at": row.created_at,
                    })
                })
                .collect();

            (StatusCode::OK, Json(serde_json::json!({
                "results": results,
                "total": total,
                "offset": offset,
                "limit": limit,
            })))
                .into_response()
        }
        Err(e) => {
            println!("[消息搜索] 查询失败: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
