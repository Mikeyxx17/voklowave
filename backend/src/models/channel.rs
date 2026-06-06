// 频道数据结构 — 数据库 channels 表映射 + 创建频道请求体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Channel {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(serde::Deserialize)]
pub struct CreateChannelInput {
    pub name: String,
}
