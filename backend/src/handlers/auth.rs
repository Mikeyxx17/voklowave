// 用户认证处理器 — 注册、登录、邮箱验证、访客登录

use crate::middleware::auth::AuthUser;
use crate::middleware::auth::Claims;
use crate::models::MeResponse;
use crate::models::VerifyEmailInput;
use crate::models::{AuthResponse, LoginInput, RegisterInput, ResendVerifyInput, User};
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use jsonwebtoken::{EncodingKey, Header, encode};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rand::RngExt;
use std::time::Duration;

/// 异步发送验证码邮件，失败不阻塞注册流程
fn send_verification_email(to_email: String, code: String) {
    tokio::spawn(async move {
        let smtp_server = std::env::var("SMTP_SERVER")
            .unwrap_or_else(|_| "smtp.office365.com".to_string());
        let username = std::env::var("SMTP_USERNAME").unwrap_or_default();
        let password = std::env::var("SMTP_PASSWORD").unwrap_or_default();

        let from_address = match format!("voklowave 验证邮件 <{}>", username).parse() {
            Ok(addr) => addr,
            Err(e) => {
                println!("❌ [邮件服务] 发件人邮箱格式解析失败: {}", e);
                return;
            }
        };
        let to_address = match to_email.parse() {
            Ok(addr) => addr,
            Err(e) => {
                println!("❌ [邮件服务] 收件人邮箱格式解析失败: {}", e);
                return;
            }
        };

        let email_content = match Message::builder()
            .from(from_address)
            .to(to_address)
            .subject("【voklowave】请激活您的团队聊天账号")
            .body(format!(
                "您好！感谢注册 voklowave。\n\n您的 6 位邮箱激活验证码为：【 {} 】\n\n该验证码在 15 分钟内有效。请尽快在客户端输入以激活您的账号。\n如果非本人操作，请忽略此邮件。",
                code
            )) {
                Ok(msg) => msg,
                Err(e) => {
                    println!("❌ [邮件服务] 构建邮件内容失败: {}", e);
                    return;
                }
            };

        let mailer_builder = match SmtpTransport::relay(&smtp_server) {
            Ok(b) => b,
            Err(e) => {
                println!("❌ [邮件服务] 连接 SMTP 服务器失败: {}", e);
                return;
            }
        };
        let mailer = mailer_builder
            .credentials(Credentials::new(username, password))
            .timeout(Some(Duration::from_secs(15)))
            .build();

        match mailer.send(&email_content) {
            Ok(_) => println!("✅ [邮件服务] 成功为用户 {} 发送验证码邮件！", to_email),
            Err(e) => println!("❌ [邮件服务] 发信被拒绝或网络超时，详细错误: {:?}", e),
        }
    });
}

pub async fn register(
    State(state): State<AppState>,
    Json(input): Json<RegisterInput>,
) -> impl IntoResponse {
    // 邮箱域名白名单校验
    let parts: Vec<&str> = input.email.split('@').collect();
    let email_domain = match parts.get(1) {
        Some(domain) => *domain,
        None => return (StatusCode::BAD_REQUEST, "邮箱格式不正确").into_response(),
    };

    let allowed_domains_str = std::env::var("ALLOWED_DOMAINS").unwrap_or_default();
    let is_allowed = allowed_domains_str
        .split(',')
        .any(|domain| domain.trim() == email_domain);

    if !is_allowed {
        return (StatusCode::FORBIDDEN, "该邮箱域名不在允许的注册白名单内").into_response();
    }

    // 输入长度校验
    if input.username.len() < 3 || input.username.len() > 30 {
        return (StatusCode::BAD_REQUEST, "用户名长度必须在 3 到 30 个字符之间").into_response();
    }
    if input.email.len() > 254 {
        return (StatusCode::BAD_REQUEST, "邮箱地址过长").into_response();
    }
    if input.password.len() < 6 || input.password.len() > 128 {
        return (StatusCode::BAD_REQUEST, "密码长度必须在 6 到 128 个字符之间").into_response();
    }

    let hashed_password = match bcrypt::hash(&input.password, 10) {
        Ok(h) => h,
        Err(e) => {
            println!("密码哈希失败: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "服务器内部错误").into_response();
        }
    };

    let verify_code = rand::rng().random_range(100000..1000000).to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(15);

    let result = sqlx::query!(
        "INSERT INTO users (username, email, password_hash, email_verify_token, token_expires_at) VALUES ($1, $2, $3, $4, $5)",
        input.username,
        input.email,
        hashed_password,
        verify_code,
        expires_at
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            send_verification_email(input.email.clone(), verify_code);
            (
                StatusCode::CREATED,
                "注册成功，请前往邮箱查收 6 位激活验证码",
            )
                .into_response()
        }
        Err(e) => {
            println!("注册失败，数据库查重未通过: {:?}", e);
            (StatusCode::CONFLICT, "用户名或邮箱已被占用").into_response()
        }
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> impl IntoResponse {
    let user_result = sqlx::query_as!(
        User,
        "SELECT id, email, password_hash, username, display_name, avatar_url, bio, is_guest, created_at, is_verified FROM users WHERE email = $1",
        input.email
    )
    .fetch_optional(&state.db)
    .await;

    match user_result {
        Ok(Some(user)) => {
            let password_ok = bcrypt::verify(&input.password, &user.password_hash).unwrap_or(false);

            if password_ok {
                // 未验证邮箱的账号拒绝登录，引导用户前往验证页面
                if !user.is_verified {
                    return (
                        StatusCode::FORBIDDEN,
                        "您的账号尚未通过邮箱验证，请先输入验证码激活",
                    )
                        .into_response();
                }

                let exp = (chrono::Utc::now() + chrono::Duration::days(7)).timestamp() as usize;

                let my_claims = Claims {
                    sub: user.id,
                    email: user.email.clone(),
                    username: user.username.clone(),
                    is_guest: false,
                    exp,
                };

                let secret_string = match std::env::var("JWT_SECRET") {
                    Ok(s) => s,
                    Err(_) => {
                        eprintln!("⚠ [安全警告] 未设置 JWT_SECRET 环境变量，使用了硬编码 fallback 密钥！请在生产环境立即配置。");
                        "development_fallback_secret_key_look_out".to_string()
                    }
                };

                let encoding_key = EncodingKey::from_secret(secret_string.as_bytes());

                let token = match encode(&Header::default(), &my_claims, &encoding_key) {
                    Ok(t) => t,
                    Err(err) => {
                        println!("JWT 签发失败: {}", err);
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                };

                let response_body = AuthResponse {
                    token,
                    username: user.username,
                    display_name: user.display_name,
                    avatar_url: user.avatar_url,
                };

                (StatusCode::OK, Json(response_body)).into_response()
            } else {
                (StatusCode::UNAUTHORIZED, "邮箱或密码错误").into_response()
            }
        }
        Ok(None) => (StatusCode::UNAUTHORIZED, "邮箱或密码错误").into_response(),
        Err(e) => {
            println!("登录查询数据库失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn get_current_user(user: AuthUser) -> impl IntoResponse {
    let response_body = MeResponse {
        id: user.user_id,
        username: user.username,
        email: user.email,
    };
    (StatusCode::OK, Json(response_body))
}

pub async fn guest_login(State(state): State<AppState>) -> impl IntoResponse {
    let unique_id = uuid::Uuid::new_v4().to_string();
    let guest_email = format!("guest_{}@temp.local", unique_id);
    let guest_username = format!("Guest_{}", &unique_id[..8]);
    let fake_password_hash = format!("g_hash_{}", unique_id);

    let user_row = sqlx::query!(
        "INSERT INTO users (username, email, password_hash, is_guest, is_verified) VALUES ($1, $2, $3, $4, true) RETURNING id",
        guest_username,
        guest_email,
        fake_password_hash,
        true
    )
    .fetch_one(&state.db)
    .await;

    match user_row {
        Ok(row) => {
            let exp = (chrono::Utc::now() + chrono::Duration::days(1)).timestamp() as usize;

            let my_claims = Claims {
                sub: row.id,
                email: guest_email,
                username: guest_username.clone(),
                is_guest: true,
                exp,
            };

            let secret_string = match std::env::var("JWT_SECRET") {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("⚠ [安全警告] 未设置 JWT_SECRET 环境变量，使用了硬编码 fallback 密钥！请在生产环境立即配置。");
                    "development_fallback_secret_key_look_out".to_string()
                }
            };
            let encoding_key = EncodingKey::from_secret(secret_string.as_bytes());

            match encode(&Header::default(), &my_claims, &encoding_key) {
                Ok(token) => {
                    let response_body = AuthResponse {
                        token,
                        username: guest_username,
                        display_name: None,
                        avatar_url: None,
                    };
                    (StatusCode::OK, Json(response_body)).into_response()
                }
                Err(err) => {
                    println!("访客 JWT 签发失败: {}", err);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
        Err(e) => {
            println!("创建访客数据库记录失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn resend_verification(
    State(state): State<AppState>,
    Json(input): Json<ResendVerifyInput>,
) -> impl IntoResponse {
    let user_row = sqlx::query!(
        "SELECT is_verified, resend_count, last_resend_at, token_expires_at FROM users WHERE email = $1",
        input.email
    )
    .fetch_optional(&state.db)
    .await;

    match user_row {
        Ok(Some(row)) => {
            if row.is_verified {
                return (StatusCode::BAD_REQUEST, "该账号已激活，无需重复发送验证码").into_response();
            }

            let now = chrono::Utc::now();
            let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();

            // 按天重置计数
            let effective_count = match row.last_resend_at {
                Some(last) if last >= today_start => row.resend_count,
                _ => 0,
            };

            if effective_count >= 3 {
                return (StatusCode::TOO_MANY_REQUESTS, "今日重发次数已用完（3 次），请明天再试").into_response();
            }

            // 60 秒冷却：如果上次验证码还剩超过 14 分钟有效期，说明刚发不久
            if let Some(exp) = row.token_expires_at {
                if exp > now + chrono::Duration::minutes(14) {
                    return (StatusCode::TOO_MANY_REQUESTS, "请等待 60 秒后再重新发送").into_response();
                }
            }

            let new_code = rand::rng().random_range(100000..1000000).to_string();
            let new_expires_at = now + chrono::Duration::minutes(15);

            match sqlx::query!(
                "UPDATE users SET email_verify_token = $1, token_expires_at = $2, resend_count = $3, last_resend_at = $4 WHERE email = $5",
                new_code,
                new_expires_at,
                effective_count + 1,
                now,
                input.email
            )
            .execute(&state.db)
            .await
            {
                Ok(_) => {
                    send_verification_email(input.email.clone(), new_code);
                    (StatusCode::OK, "验证码已重新发送，请查收邮箱").into_response()
                }
                Err(e) => {
                    println!("重新发送验证码数据库更新失败: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "操作失败，请稍后重试").into_response()
                }
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "该邮箱尚未注册").into_response(),
        Err(e) => {
            println!("查询用户失败: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn verify_email(
    State(state): State<AppState>,
    Json(input): Json<VerifyEmailInput>,
) -> impl IntoResponse {
    let user_row = sqlx::query!(
        "SELECT email_verify_token, token_expires_at, is_verified FROM users WHERE email = $1",
        input.email
    )
    .fetch_optional(&state.db)
    .await;

    match user_row {
        Ok(Some(user)) => {
            if user.is_verified {
                return (StatusCode::BAD_REQUEST, "该账号已激活，无需重复验证").into_response();
            }

            let saved_token = match user.email_verify_token {
                Some(t) => t,
                None => {
                    return (StatusCode::BAD_REQUEST, "验证码不存在，请重新注册").into_response();
                }
            };

            let now = chrono::Utc::now();
            let is_expired = user.token_expires_at.map(|exp| now > exp).unwrap_or(true);

            if is_expired {
                return (StatusCode::BAD_REQUEST, "验证码已过期，请尝试重新发送").into_response();
            }

            if saved_token != input.code {
                return (StatusCode::UNAUTHORIZED, "验证码不正确").into_response();
            }

            match sqlx::query!(
                "UPDATE users SET is_verified = true, email_verify_token = NULL, token_expires_at = NULL WHERE email = $1",
                input.email
            )
            .execute(&state.db)
            .await
            {
                Ok(_) => (StatusCode::OK, "账号激活成功！现在您可以正常登录了").into_response(),
                Err(e) => {
                    println!("激活账号数据库更新失败: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "激活失败，请稍后重试").into_response()
                }
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "该邮箱尚未注册").into_response(),
        Err(e) => {
            println!("查询验证状态失败: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
