// 请求处理器模块 — 按业务领域拆分：WebSocket / 认证 / 频道

pub mod auth;
pub mod channels;
pub mod ws;

pub use auth::{get_current_user, guest_login, login, register, resend_verification, verify_email};
pub use channels::{create_channel, get_channels};
pub use ws::ws_handler;
