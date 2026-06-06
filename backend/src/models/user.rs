// 用户数据结构 — 注册请求体 + 数据库 users 表映射

// 1. 注册接口接收的请求体
#[derive(Debug, serde::Deserialize)]
pub struct RegisterInput {
    pub username: String, // 系统唯一账号
    pub email: String,    // 登录邮箱
    pub password: String, // 登录密码
}

// 2. 数据库 users 表对应的映射结构体
#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct User {
    pub id: i32,
    pub username: String, // 账号名
    pub email: String,    // 登录邮箱
    pub password_hash: String,

    pub display_name: Option<String>, // 用户自定义的唯美昵称
    pub avatar_url: Option<String>,   //头像的网络链接
    pub bio: Option<String>,          //个性签名
    pub is_guest: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_verified: bool,
}

// 3. 登录接口接收的请求体
#[derive(Debug, serde::Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

// 用于响应登录成功的结构体
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
