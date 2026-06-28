//! 操作日志：管理员操作审计。

use crate::middleware::admin::AdminUser;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Query, State};
use axum::response::IntoResponse;

use super::AdminPagination;

/// GET /api/admin/audit-logs — 管理员操作日志（分页）
pub async fn admin_audit_logs(
    State(state): State<AppState>,
    _admin: AdminUser,
    Query(p): Query<AdminPagination>,
) -> impl IntoResponse {
    let page = p.page.max(0);
    let limit = p.limit.min(50).max(1);
    let offset = page * limit;

    let total = sqlx::query_scalar!(
        "SELECT COUNT(*)::int8 as \"count!\" FROM admin_audit_logs"
    ).fetch_one(&state.db).await.ok().unwrap_or(0);

    let rows = sqlx::query!(
        "SELECT a.id, u.username as admin_name, a.action, a.target, a.ip_address, a.created_at \
         FROM admin_audit_logs a LEFT JOIN users u ON a.admin_id = u.id \
         ORDER BY a.id DESC LIMIT $1 OFFSET $2",
        limit, offset
    ).fetch_all(&state.db).await.unwrap_or_default();

    let list: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.id,
        "admin_name": r.admin_name,
        "action": r.action,
        "target": r.target,
        "ip_address": r.ip_address,
        "created_at": r.created_at,
    })).collect();

    Json(serde_json::json!({ "logs": list, "total": total }))
}
