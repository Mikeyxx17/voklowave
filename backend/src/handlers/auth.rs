//! 认证相关接口：注册、登录、访客登录、邮箱验证、密码重置。


use crate::middleware::auth::AuthUser;
use crate::middleware::auth::Claims;
use crate::models::MeResponse;
use crate::models::VerifyEmailInput;
use crate::models::{
    AuthResponse, ForgotPasswordInput, LoginInput, RegisterInput, ResendVerifyInput,
    ResetPasswordInput, User,
};
use crate::state::AppState;
use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tracing::{error, info, warn};
use std::net::SocketAddr;
use jsonwebtoken::{EncodingKey, Header, encode};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rand::RngExt;
use std::time::Duration;

/// 异步发送验证码邮件（不阻塞当前请求）。
fn send_verification_email(to_email: String, code: String) {
    tokio::spawn(async move {
        let smtp_server =
            std::env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.qq.com".to_string());
        let username = std::env::var("SMTP_USERNAME").unwrap_or_default();
        let password = std::env::var("SMTP_PASSWORD").unwrap_or_default();

        let from_address = match format!("voklowave 验证邮件 <{}>", username).parse() {
            Ok(addr) => addr,
            Err(e) => {
                error!("邮件服务：发件人邮箱格式解析失败: {}", e);
                return;
            }
        };
        let to_address = match to_email.parse() {
            Ok(addr) => addr,
            Err(e) => {
                error!("邮件服务：收件人邮箱格式解析失败: {}", e);
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
                    error!("邮件服务：构建邮件内容失败: {}", e);
                    return;
                }
            };

        let mailer_builder = match SmtpTransport::relay(&smtp_server) {
            Ok(b) => b,
            Err(e) => {
                error!("邮件服务：连接 SMTP 服务器失败: {}", e);
                return;
            }
        };
        let mailer = mailer_builder
            .credentials(Credentials::new(username, password))
            .timeout(Some(Duration::from_secs(15)))
            .build();

        match mailer.send(&email_content) {
            Ok(_) => info!("邮件服务：成功发送验证码邮件至 {}", to_email),
            Err(e) => error!("邮件服务：发送失败: {}", e),
        }
    });
}

/// 注册新账号：校验域名白名单、输入长度、密码哈希，写入数据库并发送验证邮件。
pub async fn register(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<RegisterInput>,
) -> impl IntoResponse {
    // ── 注册限流检查 ──
    if let Err(resp) = state.register_limiter.check(addr) {
        return resp.into_response();
    }
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

    if input.username.len() < 3 || input.username.len() > 30 {
        return (
            StatusCode::BAD_REQUEST,
            "用户名长度必须在 3 到 30 个字符之间",
        )
            .into_response();
    }
    if input.email.len() > 254 {
        return (StatusCode::BAD_REQUEST, "邮箱地址过长").into_response();
    }
    if input.password.len() < 6 || input.password.len() > 128 {
        return (
            StatusCode::BAD_REQUEST,
            "密码长度必须在 6 到 128 个字符之间",
        )
            .into_response();
    }

    let hashed_password = match bcrypt::hash(&input.password, 10) {
        Ok(h) => h,
        Err(e) => {
            error!("密码哈希失败: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "服务器内部错误").into_response();
        }
    };

    let verify_code = rand::rng().random_range(100000..1000000).to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(15);

    let result = sqlx::query!(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3)",
        input.username,
        input.email,
        hashed_password,
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            let _ = sqlx::query!(
                "INSERT INTO verification_codes (email, code, purpose, expires_at) VALUES ($1, $2, 'email_verify', $3)",
                input.email,
                verify_code,
                expires_at
            )
            .execute(&state.db)
            .await;

            send_verification_email(input.email.clone(), verify_code);
            info!(
                username = %input.username,
                email = %input.email,
                "用户注册成功"
            );
            (
                StatusCode::CREATED,
                "注册成功，请前往邮箱查收 6 位激活验证码",
            )
                .into_response()
        }
        Err(e) => {
            warn!("注册失败，用户名或邮箱已被占用: {}", e);
            (StatusCode::CONFLICT, "用户名或邮箱已被占用").into_response()
        }
    }
}

/// 邮箱+密码登录：bcrypt 验密，返回 JWT（7 天有效）。未验证邮箱返回 403。
pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<LoginInput>,
) -> impl IntoResponse {
    // ── 登录限流检查 ──
    if let Err(resp) = state.login_limiter.check(addr) {
        return resp.into_response();
    }
    let user_result = sqlx::query_as!(
        User,
        "SELECT id, email, password_hash, username, display_name, avatar_url, bio, is_guest, created_at, is_verified, token_version FROM users WHERE email = $1",
        input.email
    )
    .fetch_optional(&state.db)
    .await;

    match user_result {
        Ok(Some(user)) => {
            let password_ok = bcrypt::verify(&input.password, &user.password_hash).unwrap_or(false);

            if password_ok {
                if !user.is_verified {
                    return (
                        StatusCode::FORBIDDEN,
                        "您的账号尚未通过邮箱验证，请先输入验证码激活",
                    )
                        .into_response();
                }

                let exp = (chrono::Utc::now() + chrono::Duration::days(7)).timestamp() as usize;
                let jti = uuid::Uuid::new_v4().to_string();
                let login_ip = addr.ip().to_string();

                // ── 检测其他活跃会话（不同 IP = 可能的新设备） ──
                let other_sessions = sqlx::query_scalar!(
                    "SELECT ip_address FROM sessions \
                     WHERE user_id = $1 AND is_active = TRUE AND ip_address IS NOT NULL AND ip_address != $2",
                    user.id,
                    login_ip
                )
                .fetch_all(&state.db)
                .await
                .ok()
                .unwrap_or_default();

                if !other_sessions.is_empty() {
                    warn!(
                        user_id = user.id,
                        ip = %login_ip,
                        existing_ips = ?other_sessions,
                        "检测到新 IP 登录，可能存在新设备登录"
                    );
                }

                // ── 写入会话记录 ──
                let _ = sqlx::query!(
                    "INSERT INTO sessions (user_id, jti, ip_address) VALUES ($1, $2, $3)",
                    user.id,
                    jti,
                    login_ip
                )
                .execute(&state.db)
                .await;

                let my_claims = Claims {
                    sub: user.id,
                    email: user.email.clone(),
                    username: user.username.clone(),
                    is_guest: false,
                    exp,
                    token_version: user.token_version,
                    jti,
                };

                let secret_string = std::env::var("JWT_SECRET")
                    .expect("JWT_SECRET 环境变量未设置");

                let encoding_key = EncodingKey::from_secret(secret_string.as_bytes());

                let token = match encode(&Header::default(), &my_claims, &encoding_key) {
                    Ok(t) => t,
                    Err(err) => {
                        error!("JWT 签发失败: {}", err);
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                };

                let response_body = AuthResponse {
                    token,
                    username: user.username,
                    display_name: user.display_name,
                    avatar_url: user.avatar_url,
                    is_guest: false,
                };

                info!(
                    user_id = user.id,
                    email = %user.email,
                    "用户登录成功"
                );
                (StatusCode::OK, Json(response_body)).into_response()
            } else {
                (StatusCode::UNAUTHORIZED, "邮箱或密码错误").into_response()
            }
        }
        Ok(None) => (StatusCode::UNAUTHORIZED, "账户未注册").into_response(),
        Err(e) => {
            error!("登录查询数据库失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// 获取当前登录用户完整资料（页面刷新恢复会话 + 资料字段同步）。
pub async fn get_current_user(State(state): State<AppState>, user: AuthUser) -> impl IntoResponse {
    let profile = sqlx::query!(
        "SELECT display_name, avatar_url, bio FROM users WHERE id = $1",
        user.user_id
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let response_body = MeResponse {
        id: user.user_id,
        username: user.username,
        email: user.email,
        is_guest: user.is_guest,
        display_name: profile.as_ref().and_then(|p| p.display_name.clone()),
        avatar_url: profile.as_ref().and_then(|p| p.avatar_url.clone()),
        bio: profile.as_ref().and_then(|p| p.bio.clone()),
    };
    (StatusCode::OK, Json(response_body))
}

/// 访客快速登录：生成临时账号和 JWT（1 天有效），仅限 general 频道。
pub async fn guest_login(State(state): State<AppState>) -> impl IntoResponse {
    let unique_id = uuid::Uuid::new_v4().to_string();
    let short_id = &unique_id[..8];
    let guest_email = format!("guest_{}@temp.local", short_id);
    let guest_username = format!("Guest_{}", short_id);
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
            let jti = uuid::Uuid::new_v4().to_string();

            // 写入访客会话记录
            let _ = sqlx::query!(
                "INSERT INTO sessions (user_id, jti) VALUES ($1, $2)",
                row.id,
                jti
            )
            .execute(&state.db)
            .await;

            let my_claims = Claims {
                sub: row.id,
                email: guest_email,
                username: guest_username.clone(),
                is_guest: true,
                exp,
                token_version: 1,
                jti,
            };

            let secret_string = std::env::var("JWT_SECRET")
                .expect("JWT_SECRET 环境变量未设置");
            let encoding_key = EncodingKey::from_secret(secret_string.as_bytes());

            match encode(&Header::default(), &my_claims, &encoding_key) {
                Ok(token) => {
                    let response_body = AuthResponse {
                        token,
                        username: guest_username,
                        display_name: None,
                        avatar_url: None,
                        is_guest: true,
                    };
                    (StatusCode::OK, Json(response_body)).into_response()
                }
                Err(err) => {
                    error!("访客 JWT 签发失败: {}", err);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
        Err(e) => {
            error!("创建访客数据库记录失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// 重新发送邮箱验证码（每日最多 3 次，60 秒冷却）。
pub async fn resend_verification(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<ResendVerifyInput>,
) -> impl IntoResponse {
    // ── 重发验证码限流检查 ──
    if let Err(resp) = state.resend_limiter.check(addr) {
        return resp.into_response();
    }
    let user_row = sqlx::query!(
        "SELECT is_verified FROM users WHERE email = $1",
        input.email
    )
    .fetch_optional(&state.db)
    .await;

    match user_row {
        Ok(Some(user)) => {
            if user.is_verified {
                return (StatusCode::BAD_REQUEST, "该账号已激活，无需重复发送验证码")
                    .into_response();
            }
        }
        Ok(None) => return (StatusCode::NOT_FOUND, "该邮箱尚未注册").into_response(),
        Err(e) => {
            error!("resend: 查询用户失败: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    let vc_row = sqlx::query!(
        "SELECT code, expires_at, resend_count, last_resend_at FROM verification_codes WHERE email = $1 AND purpose = 'email_verify'",
        input.email
    )
    .fetch_optional(&state.db)
    .await;

    let now = chrono::Utc::now();
    let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();

    let effective_count = match vc_row {
        Ok(Some(ref row)) => match row.last_resend_at {
            Some(last) if last >= today_start => row.resend_count,
            _ => 0,
        },
        _ => 0,
    };

    if effective_count >= 3 {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "今日重发次数已用完（3 次），请明天再试",
        )
            .into_response();
    }

    if let Ok(Some(ref row)) = vc_row {
        if row.expires_at > now + chrono::Duration::minutes(14) {
            return (StatusCode::TOO_MANY_REQUESTS, "请等待 60 秒后再重新发送").into_response();
        }
    }

    let new_code = rand::rng().random_range(100000..1000000).to_string();
    let new_expires_at = now + chrono::Duration::minutes(15);

    let upsert_result = sqlx::query!(
        "INSERT INTO verification_codes (email, code, purpose, expires_at, resend_count, last_resend_at) \
         VALUES ($1, $2, 'email_verify', $3, $4, $5) \
         ON CONFLICT (email, purpose) \
         DO UPDATE SET code = $2, expires_at = $3, resend_count = $4, last_resend_at = $5",
        input.email,
        new_code,
        new_expires_at,
        effective_count + 1,
        now
    )
    .execute(&state.db)
    .await;

    match upsert_result {
        Ok(_) => {
            send_verification_email(input.email.clone(), new_code);
            (StatusCode::OK, "验证码已重新发送，请查收邮箱").into_response()
        }
        Err(e) => {
            error!("重新发送验证码数据库更新失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "操作失败，请稍后重试").into_response()
        }
    }
}

/// 提交 6 位验证码激活邮箱（事务更新 is_verified + 删除验证码记录）。
pub async fn verify_email(
    State(state): State<AppState>,
    Json(input): Json<VerifyEmailInput>,
) -> impl IntoResponse {
    let user_row = sqlx::query!(
        "SELECT is_verified FROM users WHERE email = $1",
        input.email
    )
    .fetch_optional(&state.db)
    .await;

    match user_row {
        Ok(Some(user)) => {
            if user.is_verified {
                return (StatusCode::BAD_REQUEST, "该账号已激活，无需重复验证").into_response();
            }
        }
        Ok(None) => return (StatusCode::NOT_FOUND, "该邮箱尚未注册").into_response(),
        Err(e) => {
            error!("verify_email: 查询验证状态失败: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    let vc_row = sqlx::query!(
        "SELECT code, expires_at FROM verification_codes WHERE email = $1 AND purpose = 'email_verify'",
        input.email
    )
    .fetch_optional(&state.db)
    .await;

    let vc = match vc_row {
        Ok(Some(row)) => row,
        Ok(None) => return (StatusCode::BAD_REQUEST, "验证码不存在，请重新注册").into_response(),
        Err(e) => {
            error!("查询验证码失败: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let now = chrono::Utc::now();
    if now > vc.expires_at {
        return (StatusCode::BAD_REQUEST, "验证码已过期，请尝试重新发送").into_response();
    }

    if vc.code != input.code {
        return (StatusCode::UNAUTHORIZED, "验证码不正确").into_response();
    }

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!("verify_email: 开启事务失败: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "激活失败，请稍后重试").into_response();
        }
    };

    let user_update = sqlx::query!(
        "UPDATE users SET is_verified = true WHERE email = $1",
        input.email
    )
    .execute(&mut *tx)
    .await;

    let vc_delete = sqlx::query!(
        "DELETE FROM verification_codes WHERE email = $1 AND purpose = 'email_verify'",
        input.email
    )
    .execute(&mut *tx)
    .await;

    match (user_update, vc_delete) {
        (Ok(_), Ok(_)) => {
            if let Err(e) = tx.commit().await {
                error!("verify_email: 提交事务失败: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "激活失败，请稍后重试").into_response();
            }
            info!(email = %input.email, "邮箱验证成功");
            (StatusCode::OK, "账号激活成功！现在您可以正常登录了").into_response()
        }
        (err_user, err_vc) => {
            error!(
                ?err_user,
                ?err_vc,
                "verify_email: 激活失败"
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// 异步发送密码重置验证码邮件。
fn send_password_reset_email(to_email: String, code: String) {
    tokio::spawn(async move {
        let smtp_server =
            std::env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.qq.com".to_string());
        let username = std::env::var("SMTP_USERNAME").unwrap_or_default();
        let password = std::env::var("SMTP_PASSWORD").unwrap_or_default();

        let from_address = match format!("voklowave 安全中心 <{}>", username).parse() {
            Ok(addr) => addr,
            Err(e) => {
                error!("邮件服务：发件人邮箱格式解析失败: {}", e);
                return;
            }
        };
        let to_address = match to_email.parse() {
            Ok(addr) => addr,
            Err(e) => {
                error!("邮件服务：收件人邮箱格式解析失败: {}", e);
                return;
            }
        };

        let email_content = match Message::builder()
            .from(from_address)
            .to(to_address)
            .subject("【voklowave】密码重置验证码")
            .body(format!(
                "您正在尝试重置 voklowave 账号的密码。\n\n您的 6 位密码重置验证码为：【 {} 】\n\n该验证码在 15 分钟内有效。\n如果非本人操作，请忽略此邮件，您的密码不会改变。",
                code
            )) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("邮件服务：构建邮件内容失败: {}", e);
                    return;
                }
            };

        let mailer_builder = match SmtpTransport::relay(&smtp_server) {
            Ok(b) => b,
            Err(e) => {
                error!("邮件服务：连接 SMTP 服务器失败: {}", e);
                return;
            }
        };
        let mailer = mailer_builder
            .credentials(Credentials::new(username, password))
            .timeout(Some(Duration::from_secs(15)))
            .build();

        match mailer.send(&email_content) {
            Ok(_) => info!("邮件服务：成功发送密码重置邮件至 {}", to_email),
            Err(e) => error!("邮件服务：密码重置邮件发送失败: {}", e),
        }
    });
}

/// 发送密码重置验证码：生成 6 位数字，写入 verification_codes 表并发送邮件。
pub async fn forgot_password(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<ForgotPasswordInput>,
) -> impl IntoResponse {
    // ── 忘记密码限流检查 ──
    if let Err(resp) = state.forgot_password_limiter.check(addr) {
        return resp.into_response();
    }
    let user_row = sqlx::query!("SELECT email FROM users WHERE email = $1", input.email)
        .fetch_optional(&state.db)
        .await;

    let user_exists = match user_row {
        Ok(Some(_)) => true,
        Ok(None) => {
            return (StatusCode::OK, "如果该邮箱已注册，重置邮件已发送").into_response();
        }
        Err(e) => {
            error!("forgot_password: 查询用户失败: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !user_exists {
        return (StatusCode::OK, "如果该邮箱已注册，重置邮件已发送").into_response();
    }

    let reset_code = rand::rng().random_range(100000..1000000).to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(15);

    let upsert_result = sqlx::query!(
        "INSERT INTO verification_codes (email, code, purpose, expires_at) \
         VALUES ($1, $2, 'password_reset', $3) \
         ON CONFLICT (email, purpose) \
         DO UPDATE SET code = $2, expires_at = $3",
        input.email,
        reset_code,
        expires_at
    )
    .execute(&state.db)
    .await;

    match upsert_result {
        Ok(_) => {
            send_password_reset_email(input.email.clone(), reset_code);
            (StatusCode::OK, "如果该邮箱已注册，重置邮件已发送").into_response()
        }
        Err(e) => {
            error!("写入密码重置验证码失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// 提交验证码 + 新密码完成密码重置（事务更新 password_hash + 删除验证码记录）。
pub async fn reset_password(
    State(state): State<AppState>,
    Json(input): Json<ResetPasswordInput>,
) -> impl IntoResponse {
    if input.new_password.len() < 6 || input.new_password.len() > 128 {
        return (
            StatusCode::BAD_REQUEST,
            "新密码长度必须在 6 到 128 个字符之间",
        )
            .into_response();
    }

    let user_row = sqlx::query!("SELECT id, email FROM users WHERE email = $1", input.email)
        .fetch_optional(&state.db)
        .await;

    match user_row {
        Ok(Some(_)) => {}
        Ok(None) => return (StatusCode::NOT_FOUND, "该邮箱尚未注册").into_response(),
        Err(e) => {
            error!("resend: 查询用户失败: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    let vc_row = sqlx::query!(
        "SELECT code, expires_at FROM verification_codes WHERE email = $1 AND purpose = 'password_reset'",
        input.email
    )
    .fetch_optional(&state.db)
    .await;

    let vc = match vc_row {
        Ok(Some(row)) => row,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                "验证码不存在或已过期，请重新申请重置",
            )
                .into_response();
        }
        Err(e) => {
            error!("查询重置验证码失败: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let now = chrono::Utc::now();
    if now > vc.expires_at {
        return (StatusCode::BAD_REQUEST, "验证码已过期，请重新申请重置").into_response();
    }

    if vc.code != input.code {
        return (StatusCode::UNAUTHORIZED, "验证码不正确").into_response();
    }

    let hashed_password = match bcrypt::hash(&input.new_password, 10) {
        Ok(h) => h,
        Err(e) => {
            error!("密码哈希失败: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "服务器内部错误").into_response();
        }
    };

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!("reset_password: 开启事务失败: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "重置失败，请稍后重试").into_response();
        }
    };

    let pw_update = sqlx::query!(
        "UPDATE users SET password_hash = $1, token_version = token_version + 1 WHERE email = $2",
        hashed_password,
        input.email
    )
    .execute(&mut *tx)
    .await;

    let vc_delete = sqlx::query!(
        "DELETE FROM verification_codes WHERE email = $1 AND purpose = 'password_reset'",
        input.email
    )
    .execute(&mut *tx)
    .await;

    match (pw_update, vc_delete) {
        (Ok(_), Ok(_)) => {
            if let Err(e) = tx.commit().await {
                error!("reset_password: 提交事务失败: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "重置失败，请稍后重试").into_response();
            }
            info!(email = %input.email, "密码重置成功");
            (StatusCode::OK, "密码重置成功，请使用新密码登录").into_response()
        }
        (err_pw, err_vc) => {
            error!(
                ?err_pw,
                ?err_vc,
                "reset_password: 密码重置失败"
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}