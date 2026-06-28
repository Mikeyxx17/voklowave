//! 消息审计：全局搜索、强制删除。

use crate::middleware::admin::AdminUser;
use crate::state::{AppState, ControlEvent};
use axum::Json;
use axum::extract::{ConnectInfo, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::net::SocketAddr;
use tracing::{error, info};

use super::AdminPagination;

/// GET /api/admin/messages — 全局消息搜索（不受频道限制）
pub async fn admin_audit_messages(
    State(state): State<AppState>,
    _admin: AdminUser,
    Query(p): Query<AdminPagination>,
) -> impl IntoResponse {
    if p.q.is_empty() {
        return Json(serde_json::json!({ "messages": [], "total": 0 }));
    }

    let page = p.page.max(0);
    let limit = p.limit.min(50).max(1);
    let offset = page * limit;
    let pattern = format!("%{}%", p.q);

    let total = sqlx::query_scalar!(
        "SELECT COUNT(*)::int8 as \"count!\" FROM messages WHERE content ILIKE $1",
        pattern
    ).fetch_one(&state.db).await.ok().unwrap_or(0);

    let rows = sqlx::query!(
        "SELECT id, channel, username, content, created_at \
         FROM messages WHERE content ILIKE $1 \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        pattern, limit, offset
    ).fetch_all(&state.db).await.unwrap_or_default();

    let list: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.id, "channel": r.channel, "username": r.username,
        "content": r.content, "created_at": r.created_at,
    })).collect();

    Json(serde_json::json!({ "messages": list, "total": total }))
}

/// DELETE /api/admin/messages/{id} — 强制删除任意消息（无需是消息发送者）
pub async fn admin_force_delete_message(
    State(state): State<AppState>,
    admin: AdminUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(message_id): Path<i32>,
) -> impl IntoResponse {
    let m = sqlx::query!(
        "SELECT id, channel, username FROM messages WHERE id = $1",
        message_id
    )
    .fetch_optional(&state.db).await;

    let (channel, target_username) = match m {
        Ok(Some(ref r)) => (r.channel.clone(), r.username.clone()),
        Ok(None) => return (StatusCode::NOT_FOUND, "消息不存在").into_response(),
        Err(e) => {
            error!("查询消息失败: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!("事务失败: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let _ = sqlx::query!(
        "DELETE FROM message_reactions WHERE message_id = $1",
        message_id
    )
    .execute(&mut *tx).await;

    let _ = sqlx::query!("DELETE FROM messages WHERE id = $1", message_id)
        .execute(&mut *tx).await;

    // 写入墓碑，防止重连用户看到幽灵消息
    let _ = sqlx::query!(
        "INSERT INTO deleted_messages (id, channel) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        message_id, channel
    ).execute(&mut *tx).await;

    if let Err(e) = tx.commit().await {
        error!("提交事务失败: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let control_tx = state.get_or_create_control_channel(&channel);
    let _ = control_tx.send(ControlEvent::MessageDeleted { message_id });
    let _ = state.admin_events.send(ControlEvent::MessageDeleted { message_id });

    let _ = sqlx::query!(
        "INSERT INTO admin_audit_logs (admin_id, action, target, ip_address) VALUES ($1, 'force_delete_msg', $2, $3)",
        admin.0.user_id,
        format!("msg:{} user:{} channel:{}", message_id, target_username, channel),
        addr.ip().to_string()
    ).execute(&state.db).await;

    info!(
        admin = %admin.0.username,
        msg_id = message_id,
        target = %target_username,
        channel = %channel,
        "管理员强制删除消息"
    );
    StatusCode::NO_CONTENT.into_response()
}

/// 批量删除请求体
#[derive(serde::Deserialize)]
pub struct BatchDeleteInput {
    pub ids: Vec<i32>,
}

/// POST /api/admin/messages/batch-delete — 批量强制删除消息
pub async fn admin_batch_delete_messages(
    State(state): State<AppState>,
    admin: AdminUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<BatchDeleteInput>,
) -> impl IntoResponse {
    if input.ids.is_empty() {
        return (StatusCode::BAD_REQUEST, "ids 不能为空").into_response();
    }

    // 先将 ids 转为 Vec<i32>，避免所有权问题
    let ids = input.ids;

    // 查询所有消息（用于后续广播）
    let messages = sqlx::query!(
        "SELECT id, channel FROM messages WHERE id = ANY($1)",
        &ids
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => { error!("{}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };

    // 逐条处理
    for &msg_id in &ids {
        let _ = sqlx::query!("DELETE FROM message_reactions WHERE message_id = $1", msg_id)
            .execute(&mut *tx).await;
        let _ = sqlx::query!("DELETE FROM messages WHERE id = $1", msg_id)
            .execute(&mut *tx).await;
    }

    if let Err(e) = tx.commit().await {
        error!("批量删除事务失败: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // 广播删除事件
    for m in &messages {
        let control_tx = state.get_or_create_control_channel(&m.channel);
        let _ = control_tx.send(ControlEvent::MessageDeleted { message_id: m.id });
        let _ = state.admin_events.send(ControlEvent::MessageDeleted { message_id: m.id });
    }

    let _ = sqlx::query!(
        "INSERT INTO admin_audit_logs (admin_id, action, target, ip_address) VALUES ($1, 'batch_delete_msgs', $2, $3)",
        admin.0.user_id,
        format!("批量删除 {} 条消息", ids.len()),
        addr.ip().to_string()
    ).execute(&state.db).await;

    info!(admin = %admin.0.username, count = ids.len(), "管理员批量删除消息");
    Json(serde_json::json!({ "deleted": ids.len() })).into_response()
}
