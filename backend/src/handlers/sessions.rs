//! 会话管理：活跃会话列表 / 远程踢出。
//!
//! - GET  /api/sessions — 列出当前用户所有活跃会话
//! - DELETE /api/sessions/{id} — 停用指定会话（不是当前会话则踢掉）

use crate::middleware::auth::AuthUser;
use crate::models::Session;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tracing::{info, warn};

/// 列出当前用户所有活跃会话。
pub async fn list_sessions(
    State(state): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    let sessions = sqlx::query_as!(
        Session,
        "SELECT id, user_id, jti, ip_address, user_agent, is_active, created_at, last_seen \
         FROM sessions WHERE user_id = $1 AND is_active = TRUE \
         ORDER BY created_at DESC",
        user.user_id
    )
    .fetch_all(&state.db)
    .await;

    match sessions {
        Ok(list) => (StatusCode::OK, Json(list)).into_response(),
        Err(e) => {
            warn!("查询会话列表失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// 停用指定会话（踢出）。
/// 不能停用自己的当前会话（由 jti 判断）。
pub async fn revoke_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path(session_id): Path<i32>,
) -> impl IntoResponse {
    // 查询目标会话
    let target = sqlx::query!(
        "SELECT id, jti, user_id FROM sessions WHERE id = $1 AND is_active = TRUE",
        session_id
    )
    .fetch_optional(&state.db)
    .await;

    let target = match target {
        Ok(Some(t)) => t,
        Ok(None) => return (StatusCode::NOT_FOUND, "会话不存在或已失效").into_response(),
        Err(e) => {
            warn!("查询目标会话失败: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // 只能踢自己的会话
    if target.user_id != user.user_id {
        return (StatusCode::FORBIDDEN, "只能管理自己的会话").into_response();
    }

    let result = sqlx::query!(
        "UPDATE sessions SET is_active = FALSE WHERE id = $1",
        session_id
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            info!(
                user_id = user.user_id,
                session_id = session_id,
                "会话已被停用"
            );
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            warn!("停用会话失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
