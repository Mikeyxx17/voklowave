//! JWT 认证中间件：定义 Claims 载荷结构体，并通过 AuthUser 提取器自动校验请求令牌。

use crate::state::AppState;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

static FALLBACK_SECRET_WARNED: AtomicBool = AtomicBool::new(false);

/// JWT 令牌载荷，包含用户标识、邮箱、用户名、访客标记和过期时间。
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub email: String,
    pub username: String,
    pub is_guest: bool,
    pub exp: usize,
}

/// 通过 JWT 校验后的用户身份，handler 中直接声明该参数即可触发认证。
#[derive(Debug)]
pub struct AuthUser {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub is_guest: bool,
}

/// 从请求头 `Authorization: Bearer <token>` 中提取并校验 JWT，生成 AuthUser。
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        let auth_header = match auth_header {
            Some(header) => header,
            None => return Err(StatusCode::UNAUTHORIZED),
        };

        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let token = &auth_header[7..];

        let secret_string = match std::env::var("JWT_SECRET") {
            Ok(s) => s,
            Err(_) => {
                if !FALLBACK_SECRET_WARNED.swap(true, Ordering::Relaxed) {
                    eprintln!("⚠ [安全警告] 未设置 JWT_SECRET 环境变量，使用了硬编码 fallback 密钥！请在生产环境立即配置。");
                }
                "development_fallback_secret_key_look_out".to_string()
            }
        };

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret_string.as_bytes()),
            &Validation::default(),
        );

        match token_data {
            Ok(data) => {
                if data.claims.is_guest {
                    let exists = sqlx::query_scalar!(
                        r#"SELECT EXISTS(SELECT 1 FROM users WHERE id = $1) as "exists!""#,
                        data.claims.sub
                    )
                    .fetch_one(&state.db)
                    .await
                    .unwrap_or(false);

                    if !exists {
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                }

                Ok(AuthUser {
                    user_id: data.claims.sub,
                    username: data.claims.username,
                    email: data.claims.email,
                    is_guest: data.claims.is_guest,
                })
            }
            Err(err) => {
                println!("[验票失败] 令牌非法或已过期: {:?}", err);
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}
