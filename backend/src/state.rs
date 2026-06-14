// 全局应用状态 — 数据库连接池 + 频道内存字典 + 控制事件通道

use crate::models::ChatMessage;
use crate::services::rate_limit::RateLimiter;  // 新增
use dashmap::DashMap;
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

/// WebSocket 控制事件（消息删除等），通过独立的 broadcast channel 下发。
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ControlEvent {
    #[serde(rename = "message_deleted")]
    MessageDeleted { message_id: i32 },
    /// 表情回应切换事件（added / removed）
    #[serde(rename = "reaction_toggled")]
    ReactionToggled {
        message_id: i32,
        emoji: String,
        username: String,
        action: String, // "added" | "removed"
    },
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub channels: Arc<DashMap<String, broadcast::Sender<ChatMessage>>>,
    pub control_channels: Arc<DashMap<String, broadcast::Sender<ControlEvent>>>,
    pub login_limiter: RateLimiter,      // 新增：登录限流
    pub register_limiter: RateLimiter,   // 新增：注册限流
    pub resend_limiter: RateLimiter,     // 新增：重发验证码限流
}

impl AppState {
    pub async fn get_or_create_channel(&self, name: String) -> broadcast::Sender<ChatMessage> {
        if let Some(channel) = self.channels.get(&name) {
            return channel.clone();
        }

        sqlx::query!(
            "INSERT INTO channels (name) VALUES ($1) ON CONFLICT (name) DO NOTHING",
            name
        )
        .execute(&self.db)
        .await
        .unwrap();

        let tx = self
            .channels
            .entry(name)
            .or_insert_with(|| {
                let (new_tx, _rx) = broadcast::channel(100);
                new_tx
            })
            .clone();

        tx
    }

    /// 获取或创建频道的控制事件广播通道。
    pub fn get_or_create_control_channel(&self, name: &str) -> broadcast::Sender<ControlEvent> {
        self.control_channels
            .entry(name.to_string())
            .or_insert_with(|| {
                let (tx, _rx) = broadcast::channel(100);
                tx
            })
            .clone()
    }
}
