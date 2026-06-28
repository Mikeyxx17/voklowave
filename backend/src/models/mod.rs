//! 数据模型：用户、频道、聊天消息及其请求/响应 DTO。

pub mod channel;
pub mod dm;
pub mod message;
pub mod session;
pub mod user;

pub use channel::{Channel, CreateChannelInput};
pub use message::ChatMessage;
pub use session::Session;
pub use user::{
    AuthResponse, ForgotPasswordInput, LoginInput, MeResponse, RegisterInput,
    ResendVerifyInput, ResetPasswordInput, User, VerifyEmailInput,
};
