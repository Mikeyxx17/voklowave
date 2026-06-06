// 数据模型模块 — 所有数据结构定义、DTO、请求/响应体

pub mod channel;
pub mod message;
pub mod user;

pub use channel::{Channel, CreateChannelInput};
pub use message::ChatMessage;
pub use user::{AuthResponse, LoginInput, MeResponse, RegisterInput, User};
