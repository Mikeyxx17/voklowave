// 频道 REST API — GET /api/channels（列表）、POST /api/channels（创建）

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

    // 访客只能看到 general 频道
    if user.is_guest {
        let filtered_channels: Vec<Channel> = all_channels
            .into_iter()
            .filter(|c| c.name == "general")
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
            // 在内存中同步创建该频道的广播通道
            let (tx, _rx) = tokio::sync::broadcast::channel(100);
            state.channels.insert(input.name.clone(), tx);

            println!(
                "用户 {} 创建频道 {} 成功，当前共 {} 个频道",
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
