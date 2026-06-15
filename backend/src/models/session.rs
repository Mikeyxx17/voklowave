//! 会话数据结构：`sessions` 表的 ORM 映射。

/// 数据库 `sessions` 表的一行。
#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct Session {
    pub id: i32,
    pub user_id: i32,
    pub jti: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}
