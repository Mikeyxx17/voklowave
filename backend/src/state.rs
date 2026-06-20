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
    /// 新消息创建事件（管理后台实时更新用）
    #[serde(rename = "message_created")]
    MessageCreated {
        message_id: i32,
        channel: String,
        username: String,
    },
    /// 新频道创建事件（管理后台实时更新用）
    #[serde(rename = "channel_created")]
    ChannelCreated { name: String },
    /// 频道被删除事件（管理后台实时更新用）
    #[serde(rename = "channel_deleted")]
    ChannelDeleted { name: String },
    /// 新用户注册/访客登录事件（管理后台实时更新用）
    #[serde(rename = "user_created")]
    UserCreated { username: String },
    /// 用户被删除事件（管理后台实时更新用）
    #[serde(rename = "user_deleted")]
    UserDeleted { user_id: i32 },
    /// 用户管理员身份切换事件（管理后台实时更新用）
    #[serde(rename = "user_admin_toggled")]
    UserAdminToggled { user_id: i32 },
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
    /// 全局管理后台事件通道（消息创建/删除等，推送给 admin 页面实时刷新）
    pub admin_events: broadcast::Sender<ControlEvent>,
    /// 全局用户事件通道（UserDeleted 等推送给所有在线客户端，驱动前端自动登出）
    pub global_events: broadcast::Sender<ControlEvent>,
    pub login_limiter: RateLimiter,
    pub register_limiter: RateLimiter,
    pub resend_limiter: RateLimiter,
    pub forgot_password_limiter: RateLimiter,
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
