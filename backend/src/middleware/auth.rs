//! JWT 认证中间件：定义 Claims 载荷结构体，并通过 AuthUser 提取器自动校验请求令牌。

use crate::state::AppState;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use tracing::warn;

/// JWT 令牌载荷，包含用户标识、邮箱、用户名、访客标记和过期时间。
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub email: String,
    pub username: String,
    pub is_guest: bool,
    pub exp: usize,
    /// 签发时的 token 版本号，改密码后递增，旧 token 自动失效
    pub token_version: i32,
    /// JWT 唯一标识（jti），用于会话管理和踢出
    pub jti: String,
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

        let secret_string = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET 环境变量未设置");

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret_string.as_bytes()),
            &Validation::default(),
        );

        match token_data {
            Ok(data) => {
                // 校验 token_version：对比数据库当前版本，不一致则令牌已失效（密码已改）
                let current_version = sqlx::query_scalar!(
                    "SELECT token_version FROM users WHERE id = $1",
                    data.claims.sub
                )
                .fetch_optional(&state.db)
                .await
                .ok()
                .flatten();

                match current_version {
                    None => return Err(StatusCode::UNAUTHORIZED),
                    Some(db_version) if db_version != data.claims.token_version => {
                        warn!(
                            user_id = data.claims.sub,
                            token_ver = data.claims.token_version,
                            db_ver = db_version,
                            "Token 版本不匹配，令牌已失效"
                        );
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                    _ => {}
                }

                // 校验会话是否仍然有效（被踢出的会话 is_active = false）
                let session_active = sqlx::query_scalar!(
                    "SELECT is_active FROM sessions WHERE jti = $1",
                    data.claims.jti
                )
                .fetch_optional(&state.db)
                .await
                .ok()
                .flatten();

                if session_active == Some(false) {
                    warn!(
                        user_id = data.claims.sub,
                        jti = %data.claims.jti,
                        "会话已被踢出"
                    );
                    return Err(StatusCode::UNAUTHORIZED);
                }

                if data.claims.is_guest {
                    // 访客额外检查：是否已被清理
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
                warn!("JWT 验票失败: {}", err);
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}
