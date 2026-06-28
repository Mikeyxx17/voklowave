//! 仪表盘：全站概览统计。

use crate::middleware::admin::AdminUser;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;

/// GET /api/admin/dashboard — 全站概览统计
pub async fn admin_dashboard(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> impl IntoResponse {
    let total_users = sqlx::query_scalar!(
        "SELECT COUNT(*)::int8 as \"count!\" FROM users"
    ).fetch_one(&state.db).await.ok().unwrap_or(0);
    let total_messages = sqlx::query_scalar!(
        "SELECT COUNT(*)::int8 as \"count!\" FROM messages"
    ).fetch_one(&state.db).await.ok().unwrap_or(0);
    let total_channels = sqlx::query_scalar!(
        "SELECT COUNT(*)::int8 as \"count!\" FROM channels"
    ).fetch_one(&state.db).await.ok().unwrap_or(0);
    let today_messages = sqlx::query_scalar!(
        "SELECT COUNT(*)::int8 as \"count!\" FROM messages WHERE created_at > NOW() - INTERVAL '24 hours'"
    ).fetch_one(&state.db).await.ok().unwrap_or(0);
    let guest_count = sqlx::query_scalar!(
        "SELECT COUNT(*)::int8 as \"count!\" FROM users WHERE is_guest = true"
    ).fetch_one(&state.db).await.ok().unwrap_or(0);

    Json(serde_json::json!({
        "total_users": total_users,
        "total_messages": total_messages,
        "total_channels": total_channels,
        "today_messages": today_messages,
        "guest_count": guest_count,
    }))
}
