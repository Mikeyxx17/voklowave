//! 聊天消息数据结构：数据库 `messages` 表的 ORM 映射，兼作 WebSocket 收发格式。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 一条聊天消息（客户端发送时无 id、created_at；数据库回放时两者均有值）。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChatMessage {
    pub id: Option<i32>,
    pub channel: String,
    pub username: String,
    pub content: String,
    pub created_at: Option<DateTime<Utc>>,

    #[sqlx(default)]
    pub display_name: Option<String>,
    #[sqlx(default)]
    pub avatar_url: Option<String>,
}
