//! 用户数据结构：注册/登录请求体、数据库行映射、认证响应。

/// 注册请求体。
#[derive(Debug, serde::Deserialize)]
pub struct RegisterInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// 数据库 `users` 表的行映射。
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
    pub is_admin: bool,
    pub is_superadmin: bool,
    pub is_owner: bool,
    pub token_version: i32,
    pub muted_until: Option<chrono::DateTime<chrono::Utc>>,
}

/// 登录请求体。
#[derive(Debug, serde::Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

/// 登录/注册成功后返回的 JWT + 用户信息。
#[derive(Debug, serde::Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_guest: bool,
    pub is_admin: bool,
    pub is_owner: bool,
}

/// `GET /api/me` 响应：当前用户完整资料。
#[derive(serde::Serialize)]
pub struct MeResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub is_guest: bool,
    pub is_admin: bool,
    pub is_superadmin: bool,
    pub is_owner: bool,
    pub muted_until: Option<chrono::DateTime<chrono::Utc>>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

/// 邮箱验证请求体。
#[derive(Debug, serde::Deserialize)]
pub struct VerifyEmailInput {
    pub email: String,
    pub code: String,
}

/// 重发验证码请求体。
#[derive(Debug, serde::Deserialize)]
pub struct ResendVerifyInput {
    pub email: String,
}

/// 忘记密码请求体。
#[derive(Debug, serde::Deserialize)]
pub struct ForgotPasswordInput {
    pub email: String,
}

/// 重置密码请求体。
#[derive(Debug, serde::Deserialize)]
pub struct ResetPasswordInput {
    pub email: String,
    pub code: String,
    pub new_password: String,
}
