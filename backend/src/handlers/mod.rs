//! HTTP 请求处理器：认证、频道、WebSocket、资料编辑、消息管理、管理员后台。

pub mod admin;
pub mod auth;
pub mod channels;
pub mod messages;
pub mod profile;
pub mod reactions;
pub mod sessions;
pub mod ws;

pub use admin::{
    admin_audit_logs, admin_audit_messages, admin_dashboard, admin_delete_channel,
    admin_delete_user, admin_force_delete_message, admin_list_channels, admin_list_users,
    admin_toggle_admin,
};
pub use auth::{
    forgot_password, get_current_user, guest_login, login, register, resend_verification,
    reset_password, verify_email,
};
pub use channels::{create_channel, get_channels};
pub use messages::{delete_message, search_messages};
pub use profile::{list_users, update_profile};
pub use reactions::toggle_reaction;
pub use sessions::{list_sessions, revoke_session};
pub use ws::{admin_ws_handler, ws_handler};
