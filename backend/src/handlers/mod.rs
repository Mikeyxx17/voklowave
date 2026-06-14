//! HTTP 请求处理器：认证、频道、WebSocket、资料编辑、消息管理。

pub mod auth;
pub mod channels;
pub mod messages;   // 新增：消息硬删除（含墓碑表）
pub mod profile;    // 新增：用户资料编辑
pub mod reactions;  // 新增：表情回应
pub mod ws;

pub use auth::{
    forgot_password, get_current_user, guest_login, login, register, resend_verification,
    reset_password, verify_email,
};
pub use channels::{create_channel, get_channels};
pub use messages::{delete_message, search_messages};    // 新增
pub use profile::{list_users, update_profile};     // 新增
pub use reactions::toggle_reaction;  // 新增
pub use ws::ws_handler;
