// src/auth.rs

use crate::state::AppState;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

static FALLBACK_SECRET_WARNED: AtomicBool = AtomicBool::new(false);

// 1. 定义与登录签发时完全一致的 JWT 荷载载荷 (Payload)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,         // 用户数据库 ID
    pub email: String,    // 用户唯一邮箱
    pub username: String, // 用户唯一账号名
    pub is_guest: bool,   // 🌟 新增：让门票自带访客属性
    pub exp: usize,       // 门票截止（过期）时间戳
}

// 2. 定义我们的自定义"检票员"结构体
// 只要在别的处理器函数参数里写上 (AuthUser: AuthUser)，就代表该接口必须登录！
#[derive(Debug)]
pub struct AuthUser {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub is_guest: bool, // 🌟 新增：让门票自带访客属性
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // 步骤一：去 HTTP 请求头的"车头"格子里寻找 Authorization [cite: 266, 270]
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        // 如果连门票都没带，直接冷酷拦截，抛出 401 未登录错误 [cite: 271, 274]
        let auth_header = match auth_header {
            Some(header) => header,
            None => return Err(StatusCode::UNAUTHORIZED),
        };

        // 步骤二：检查门票格式是否以 "Bearer " 开头
        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }

        // 切掉前7个字符，抓取到最核心的 JWT 密文长字符串
        let token = &auth_header[7..];

        // 步骤三：动态从环境变量读取我们的独家防伪密钥 [cite: 232]
        let secret_string = match std::env::var("JWT_SECRET") {
            Ok(s) => s,
            Err(_) => {
                if !FALLBACK_SECRET_WARNED.swap(true, Ordering::Relaxed) {
                    eprintln!("⚠ [安全警告] 未设置 JWT_SECRET 环境变量，使用了硬编码 fallback 密钥！请在生产环境立即配置。");
                }
                "development_fallback_secret_key_look_out".to_string()
            }
        };

        // 步骤四：开始利用 jsonwebtoken 库对门票进行真伪检测和过期核验 [cite: 222, 232]
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret_string.as_bytes()), // 传入我们的解密钥匙
            &Validation::default(), // 校验配置（默认会自动验证 exp 是否过期） [cite: 222]
        );

        // 步骤五：根据验票结果进行放行或拦截
        match token_data {
            Ok(data) => {
                // 仅对访客做 DB 存活检查：访客可能已被后台清理任务删除
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
                println!("[验票失败] 门票非法或已过期: {:?}", err);
                // 🎟️ 假票或者过期票，拦截！丢回 401 状态码 [cite: 222, 274]
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}
