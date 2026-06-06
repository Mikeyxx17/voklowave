// JWT 认证中间件 — 从 Authorization 头提取并校验令牌，注入 AuthUser

use crate::state::AppState;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

static FALLBACK_SECRET_WARNED: AtomicBool = AtomicBool::new(false);

/// JWT 载荷 — 与登录签发时的字段保持一致
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub email: String,
    pub username: String,
    pub is_guest: bool,
    pub exp: usize,
}

/// 通过认证的用户信息，在 handler 参数中声明即可触发 JWT 校验
#[derive(Debug)]
pub struct AuthUser {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub is_guest: bool,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
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
                // 访客需要额外检查 DB 存活状态（可能已被定时清理任务删除）
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
            },
            Err(err) => {
                println!("[验票失败] 令牌非法或已过期: {:?}", err);
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}
