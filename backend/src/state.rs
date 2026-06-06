// 全局应用状态 — 数据库连接池 + 频道内存字典

use crate::models::ChatMessage;
use dashmap::DashMap;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub channels: Arc<DashMap<String, broadcast::Sender<ChatMessage>>>,
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
}
