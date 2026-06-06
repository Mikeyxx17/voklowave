// 用户数据结构 — 注册/登录请求体、数据库 users 表映射、JWT 响应

#[derive(Debug, serde::Deserialize)]
pub struct RegisterInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,

    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub is_guest: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_verified: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(serde::Serialize)]
pub struct MeResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct VerifyEmailInput {
    pub email: String,
    pub code: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ResendVerifyInput {
    pub email: String,
}
