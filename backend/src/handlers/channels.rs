//! 频道 REST API：提供频道列表查询和新频道创建功能。

use crate::middleware::auth::AuthUser;
use crate::models::{Channel, CreateChannelInput};
use crate::state::{AppState, ControlEvent};
use axum::Json;
use tracing::{error, info, warn};
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse};

/// 获取当前用户可见的频道列表（访客仅能看见 general）。
pub async fn get_channels(State(state): State<AppState>, user: AuthUser) -> Json<Vec<Channel>> {
    let all_channels = sqlx::query_as::<_, Channel>("SELECT * FROM channels")
        .fetch_all(&state.db)
        .await
        .unwrap();

    if user.is_guest {
        let filtered_channels: Vec<Channel> = all_channels
            .into_iter()
            .filter(|c| c.name == "general")
            .collect();
        return Json(filtered_channels);
    }

    Json(all_channels)
}

/// 创建新频道：持久化到数据库并在内存中注册广播通道（访客禁止）。
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
        Ok(r) => {
            // ── 检查是否实际插入了行（rows_affected = 0 意味着名字已存在） ──
            if r.rows_affected() == 0 {
                warn!(channel = %input.name, user = %user.username, "频道名已被占用");
                return (StatusCode::CONFLICT, "该频道名称已被使用").into_response();
            }

            state.channels.entry(input.name.clone()).or_insert_with(|| {
                let (tx, _rx) = tokio::sync::broadcast::channel(100);
                tx
            });
            // 同时创建控制事件通道
            state.get_or_create_control_channel(&input.name);

            // 广播给管理后台实时刷新
            let _ = state.admin_events.send(ControlEvent::ChannelCreated { name: input.name.clone() });

            info!(
                user = %user.username,
                channel = %input.name,
                total = state.channels.len(),
                "频道创建成功"
            );

            StatusCode::CREATED.into_response()
        }
        Err(e) => {
            error!("用户 {} 创建频道 {} 失败: {}", user.username, input.name, e);
            StatusCode::BAD_REQUEST.into_response()
        }
    }
}
