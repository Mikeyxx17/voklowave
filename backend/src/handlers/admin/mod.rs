//! 管理员 API 路由注册与通用分页参数。

mod dashboard;
mod users;
mod channels;
mod messages;
mod audit_logs;

use axum::{
    Router,
    routing::{delete, get, patch, post},
};
use crate::state::AppState;
use serde::Deserialize;

pub use dashboard::admin_dashboard;
pub use users::{admin_batch_delete_users, admin_batch_toggle_admin, admin_delete_user, admin_list_users, admin_mute_user, admin_toggle_admin};
pub use channels::{admin_delete_channel, admin_list_channels};
pub use messages::{admin_audit_messages, admin_batch_delete_messages, admin_force_delete_message};
pub use audit_logs::admin_audit_logs;

/// 通用分页参数
#[derive(Deserialize)]
pub struct AdminPagination {
    #[serde(default)]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub limit: i64,
    #[serde(default)]
    pub q: String,
}

fn default_page_size() -> i64 { 20 }

/// 注册所有管理后台路由
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(admin_dashboard))
        .route("/users", get(admin_list_users))
        .route("/users/{id}", delete(admin_delete_user))
        .route("/users/{id}/toggle-admin", patch(admin_toggle_admin))
        .route("/users/{id}/mute", patch(admin_mute_user))
        .route("/users/batch-delete", post(admin_batch_delete_users))
        .route("/users/batch-toggle-admin", post(admin_batch_toggle_admin))
        .route("/channels", get(admin_list_channels))
        .route("/channels/{id}", delete(admin_delete_channel))
        .route("/messages", get(admin_audit_messages))
        .route("/messages/{id}", delete(admin_force_delete_message))
        .route("/messages/batch-delete", post(admin_batch_delete_messages))
        .route("/audit-logs", get(admin_audit_logs))
        .with_state(state)
}
