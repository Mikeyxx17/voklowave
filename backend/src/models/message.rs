// 聊天消息数据结构 — WebSocket 消息收发 + 数据库 messages 表映射

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChatMessage {
    pub id: Option<i32>,
    pub channel: String,
    pub username: String, // 对应发送者的系统唯一账号 username
    pub content: String,
    pub created_at: Option<DateTime<Utc>>,

    #[sqlx(default)]
    pub display_name: Option<String>,
    #[sqlx(default)]
    pub avatar_url: Option<String>,
}
