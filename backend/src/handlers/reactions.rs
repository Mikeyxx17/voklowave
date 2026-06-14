//! 表情回应：POST /api/messages/{id}/react — 切换表情反应（添加/移除）。
//!
//! 逻辑：
//!   - INSERT ... ON CONFLICT DO NOTHING
//!   - rows_affected == 1 → 新增成功 → 广播 "added"
//!   - rows_affected == 0 → 已存在 → DELETE → 广播 "removed"

use crate::middleware::auth::AuthUser;
use crate::state::{AppState, ControlEvent};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

/// 请求体：要切换的表情。
#[derive(serde::Deserialize)]
pub struct ReactInput {
    pub emoji: String,
}

/// 响应体。
#[derive(serde::Serialize)]
pub struct ReactResponse {
    pub action: String,      // "added" | "removed"
    pub message_id: i32,
    pub emoji: String,
    pub username: String,
}

/// 切换对消息的表情反应：点一次添加，再点一次移除。
pub async fn toggle_reaction(
    State(state): State<AppState>,
    user: AuthUser,
    Path(message_id): Path<i32>,
    Json(input): Json<ReactInput>,
) -> impl IntoResponse {
    // ── 检查消息是否存在 + 获取频道名（用于广播） ──
    let msg = sqlx::query!(
        "SELECT id, channel FROM messages WHERE id = $1",
        message_id
    )
    .fetch_optional(&state.db)
    .await;

    let channel = match msg {
        Ok(Some(m)) => m.channel,
        Ok(None) => return (StatusCode::NOT_FOUND, "消息不存在").into_response(),
        Err(e) => {
            println!("[表情回应] 查询消息失败: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // ── 尝试 INSERT（靠 UNIQUE 约束防重复） ──
    let insert_result = sqlx::query!(
        "INSERT INTO message_reactions (message_id, username, emoji) \
         VALUES ($1, $2, $3) ON CONFLICT (message_id, username, emoji) DO NOTHING",
        message_id,
        user.username,
        input.emoji
    )
    .execute(&state.db)
    .await;

    match insert_result {
        Ok(r) => {
            if r.rows_affected() == 1 {
                // ── 新增成功：广播 added ──
                let control_tx = state.get_or_create_control_channel(&channel);
                let _ = control_tx.send(ControlEvent::ReactionToggled {
                    message_id,
                    emoji: input.emoji.clone(),
                    username: user.username.clone(),
                    action: "added".into(),
                });

                (StatusCode::OK, Json(ReactResponse {
                    action: "added".into(),
                    message_id,
                    emoji: input.emoji,
                    username: user.username,
                })).into_response()
            } else {
                // ── 已存在（rows_affected == 0）：DELETE 移除 ──
                let _ = sqlx::query!(
                    "DELETE FROM message_reactions \
                     WHERE message_id = $1 AND username = $2 AND emoji = $3",
                    message_id,
                    user.username,
                    input.emoji
                )
                .execute(&state.db)
                .await;

                let control_tx = state.get_or_create_control_channel(&channel);
                let _ = control_tx.send(ControlEvent::ReactionToggled {
                    message_id,
                    emoji: input.emoji.clone(),
                    username: user.username.clone(),
                    action: "removed".into(),
                });

                (StatusCode::OK, Json(ReactResponse {
                    action: "removed".into(),
                    message_id,
                    emoji: input.emoji,
                    username: user.username,
                })).into_response()
            }
        }
        Err(e) => {
            println!("[表情回应] INSERT 失败: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
