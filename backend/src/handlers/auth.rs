// 用户认证处理器 — 注册、登录

use crate::middleware::auth::AuthUser;
use crate::middleware::auth::Claims;
use crate::models::MeResponse;
use crate::models::{AuthResponse, LoginInput, RegisterInput, User};
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use jsonwebtoken::{EncodingKey, Header, encode};

//用户注册函数
pub async fn register(
    State(state): State<AppState>,
    Json(input): Json<RegisterInput>,
) -> impl IntoResponse {
    let hashed_password = bcrypt::hash(&input.password, 10).unwrap();

    let result = sqlx::query!(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3)",
        input.username,
        input.email,
        hashed_password
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => StatusCode::CREATED,
        Err(_) => StatusCode::CONFLICT,
    }
}

// 用户登录函数
pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> impl IntoResponse {
    let user_result = sqlx::query_as!(
        User,
        "SELECT id, email, password_hash, username, display_name, avatar_url, bio, is_guest, created_at FROM users WHERE email = $1",
        input.email
    )
    .fetch_optional(&state.db)
    .await;
    match user_result {
        Ok(Some(user)) => {
            let password_ok = bcrypt::verify(&input.password, &user.password_hash).unwrap_or(false);

            if password_ok {
                let exp = (chrono::Utc::now() + chrono::Duration::days(7)).timestamp() as usize;

                let my_claims = Claims {
                    sub: user.id,
                    email: user.email.clone(),
                    username: user.username.clone(),
                    is_guest: false,
                    exp,
                };
                // -------------------------------------------------------------------------------------
                let secret_string = std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "development_fallback_secret_key_look_out".to_string());

                let encoding_key = EncodingKey::from_secret(secret_string.as_bytes());

                let token = match encode(&Header::default(), &my_claims, &encoding_key) {
                    Ok(t) => t,
                    Err(err) => {
                        println!("JWT 签发失败: {}", err);
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                };
                //--------------------------------------------------------------------------------------------

                let response_body = AuthResponse {
                    token,
                    username: user.username,
                    display_name: user.display_name,
                    avatar_url: user.avatar_url,
                };

                // 返回 200 OK 并附带 JSON 数据
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

    // 1. 往数据库插入访客记录并拿到自动生成的 ID
    let user_row = sqlx::query!(
        "INSERT INTO users (username, email, password_hash, is_guest) VALUES ($1, $2, $3, $4) RETURNING id",
        guest_username,
        guest_email,
        fake_password_hash,
        true
    )
    .fetch_one(&state.db)
    .await;

    match user_row {
        Ok(row) => {
            // 2. 计算过期时间（比如临时访客证只给 1 天有效期）
            let exp = (chrono::Utc::now() + chrono::Duration::days(1)).timestamp() as usize;

            // 3. 构造 Claims 门票信息
            let my_claims = Claims {
                sub: row.id, // 🌟 顺利拿到刚生成的访客 ID
                email: guest_email,
                username: guest_username.clone(),
                is_guest: true,
                exp,
            };

            // 4. 读取防伪钢印密钥
            let secret_string = std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "development_fallback_secret_key_look_out".to_string());
            let encoding_key = EncodingKey::from_secret(secret_string.as_bytes());

            // 5. 签发 Token
            match encode(&Header::default(), &my_claims, &encoding_key) {
                Ok(token) => {
                    let response_body = AuthResponse {
                        token,
                        username: guest_username,
                        display_name: None, // 访客暂时没有自定义昵称
                        avatar_url: None,   // 访客暂时没有头像
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
