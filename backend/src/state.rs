//! 全局应用状态：持有 PostgreSQL 连接池和频道内存广播字典。

use crate::models::ChatMessage;
use dashmap::DashMap;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

/// 应用全局共享状态，axum 通过 `State` 提取器注入到每个 handler。
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接池
    pub db: PgPool,
    /// 频道名 → broadcast::Sender 的并发安全映射
    pub channels: Arc<DashMap<String, broadcast::Sender<ChatMessage>>>,
}

impl AppState {
    /// 获取或创建指定频道的广播发送器：先查内存缓存，未命中则持久化到数据库并创建新通道。
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
}
