// 频道 REST API 处理器 — GET/POST /api/channels

use crate::middleware::auth::AuthUser;
use crate::models::{Channel, CreateChannelInput};
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse};

pub async fn get_channels(State(state): State<AppState>, user: AuthUser) -> Json<Vec<Channel>> {
    let all_channels = sqlx::query_as::<_, Channel>("SELECT * FROM channels")
        .fetch_all(&state.db)
        .await
        .unwrap();

    if user.is_guest {
        let filtered_channels: Vec<Channel> = all_channels
            .into_iter()
            .filter(|c| c.name == "general") // 只保留名字是 "general" 的频道
            .collect();
        return Json(filtered_channels);
    }

    Json(all_channels)
}

pub async fn create_channel(
    State(state): State<AppState>,
    user: AuthUser,
    Json(input): Json<CreateChannelInput>,
) -> impl IntoResponse {
    // 🌟 核心防线：如果当前用户是访客，并且他尝试访问非 general 频道，直接返回 403
    if user.is_guest {
        return (StatusCode::FORBIDDEN, "访客模式下无法创建新频道").into_response();
    }
    let result = sqlx::query!(
        "INSERT INTO channels (name) VALUES ($1) ON CONFLICT (name) DO NOTHING",
        input.name
    )
    .execute(&state.db)
    .await;
    match result {
        Ok(_) => {
            // 5. 频道创建成功后，别忘了动态在内存里为这个新频道开通一个广播通道
            let (tx, _rx) = tokio::sync::broadcast::channel(100);
            state.channels.insert(input.name.clone(), tx);

            println!(
                "用户 {} 创建频道 {} 成功。\n现在有 {} 个频道",
                user.username,
                input.name,
                state.channels.len()
            );

            StatusCode::CREATED.into_response()
        }
        Err(e) => {
            println!("用户 {} 创建频道 {} 失败: {}", user.username, input.name, e);
            StatusCode::BAD_REQUEST.into_response()
        }
    }
}
