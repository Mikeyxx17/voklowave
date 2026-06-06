// WebSocket 聊天消息处理器 — 连接升级、生命周期管理、消息广播

use crate::middleware::auth::{AuthUser, Claims};
use crate::models::ChatMessage;
use crate::state::AppState;
use axum::extract::{
    Path, Query, WebSocketUpgrade as Ws,
    ws::{Message, WebSocket},
};
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse};
use futures::{sink::SinkExt, stream::StreamExt};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::sync::atomic::{AtomicBool, Ordering};

static FALLBACK_SECRET_WARNED: AtomicBool = AtomicBool::new(false);

#[derive(serde::Deserialize)]
pub(crate) struct WsQuery {
    pub token: Option<String>,
}

pub async fn ws_handler(
    ws: Ws,
    Path(channel_name): Path<String>,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
) -> impl IntoResponse {
    let token = match query.token {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "缺少认证令牌").into_response(),
    };

    let secret_string = match std::env::var("JWT_SECRET") {
        Ok(s) => s,
        Err(_) => {
            if !FALLBACK_SECRET_WARNED.swap(true, Ordering::Relaxed) {
                eprintln!("⚠ [安全警告] 未设置 JWT_SECRET 环境变量，使用了硬编码 fallback 密钥！请在生产环境立即配置。");
            }
            "development_fallback_secret_key_look_out".to_string()
        }
    };

    let token_data = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret_string.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("[WS验票失败] 门票非法或已过期: {:?}", err);
            return (StatusCode::UNAUTHORIZED, "认证令牌无效或已过期").into_response();
        }
    };

    let user = AuthUser {
        user_id: token_data.claims.sub,
        username: token_data.claims.username,
        email: token_data.claims.email,
        is_guest: token_data.claims.is_guest,
    };

    if user.is_guest {
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE id = $1) as "exists!""#,
            user.user_id
        )
        .fetch_one(&state.db)
        .await
        .unwrap_or(false);

        if !exists {
            return (StatusCode::UNAUTHORIZED, "访客账号已失效").into_response();
        }
        if channel_name != "general" {
            return (StatusCode::FORBIDDEN, "访客只能访问 general 频道").into_response();
        }
    }

    ws.on_upgrade(|socket| handle_socket(socket, state, channel_name, user))
}

async fn handle_socket(socket: WebSocket, state: AppState, channel_name: String, user: AuthUser) {
    let tx = state.get_or_create_channel(channel_name.clone()).await;

    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();

    let history = sqlx::query_as::<_, ChatMessage>(
        "SELECT * FROM (SELECT * FROM messages WHERE channel = $1 ORDER BY id DESC LIMIT 50) AS sub ORDER BY id ASC",
    )
    .bind(&channel_name)
    .fetch_all(&state.db)
    .await;

    if let Ok(msgs) = history {
        for msg in msgs {
            if let Ok(json) = serde_json::to_string(&msg) {
                if let Err(_) = sender.send(Message::Text(json.into())).await {
                    return;
                }
            }
        }
    }

    loop {
        tokio::select! {
            user_msg = receiver.next() => {
                match user_msg {
                    Some(Ok(msg)) => {
                        if let Message::Text(text) = msg {
                            // 心跳：收到 ping 回 pong，跳过正常的消息解析
                            if let Ok(ping) = serde_json::from_str::<serde_json::Value>(&text) {
                                if ping.get("type").and_then(|v| v.as_str()) == Some("ping") {
                                    let _ = sender.send(Message::Text("{\"type\":\"pong\"}".into())).await;
                                    continue;
                                }
                            }
                            match serde_json::from_str::<ChatMessage>(&text) {
                                Ok(mut parsed_msg) => {
                                    parsed_msg.channel = channel_name.clone();
                                    parsed_msg.username = user.username.clone();
                                    parsed_msg.display_name = None;
                                    parsed_msg.avatar_url = None;

                                    let db_result = sqlx::query(
                                        "INSERT INTO messages (channel, username, content) VALUES ($1, $2, $3)"
                                    )
                                    .bind(&parsed_msg.channel)
                                    .bind(&parsed_msg.username)
                                    .bind(&parsed_msg.content)
                                    .execute(&state.db)
                                    .await;

                                    if let Ok(_) = db_result {
                                        parsed_msg.created_at = Some(chrono::Utc::now());
                                        let _ = tx.send(parsed_msg);
                                    } else if let Err(e) = db_result {
                                        println!("数据库写入失败: {}", e);
                                    }
                                }
                                Err(e) => println!("解析失败: {}", e),
                            }
                        }
                    }
                    _ => break,
                }
            }

            recv_result = rx.recv() => {
                if let Ok(broadcast_msg) = recv_result {
                    if let Ok(json_text) = serde_json::to_string(&broadcast_msg) {
                        if let Err(_) = sender.send(Message::Text(json_text.into())).await {
                            break;
                        }
                    }
                }
            }
        }
    }
}
