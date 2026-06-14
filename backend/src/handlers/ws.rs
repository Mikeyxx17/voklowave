//! WebSocket 实时消息处理：JWT 校验、协议升级、消息广播、历史回放、心跳保活。
//!
//! 新增：控制事件通道监听（消息删除等），墓碑表同步。

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

/// WebSocket 连接 URL 查询参数（`?token=`）。
#[derive(serde::Deserialize)]
pub(crate) struct WsQuery {
    pub token: Option<String>,
}

/// WebSocket 升级入口：校验 JWT、检查访客权限，升级成功后进入消息循环。
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

/// WebSocket 主循环：
///   1. 下发墓碑列表（最近 1 小时被删的消息 ID）
///   2. 回放最近 50 条历史消息
///   3. 三路 select：用户消息 / 聊天广播 / 控制事件广播
async fn handle_socket(socket: WebSocket, state: AppState, channel_name: String, user: AuthUser) {
    let tx = state.get_or_create_channel(channel_name.clone()).await;
    let control_tx = state.get_or_create_control_channel(&channel_name);  // 新增

    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();
    let mut crx = control_tx.subscribe();  // 新增：控制事件订阅

    // ── 1. 下发墓碑列表（防止重连后遗留幽灵消息） ──
    let deletions = sqlx::query_scalar!(
        "SELECT id FROM deleted_messages WHERE channel = $1 AND deleted_at > NOW() - INTERVAL '1 hour'",
        channel_name
    )
    .fetch_all(&state.db)
    .await;

    if let Ok(ids) = deletions {
        for id in ids {
            let event = serde_json::json!({"type":"message_deleted","message_id":id});
            if sender.send(Message::Text(event.to_string().into())).await.is_err() {
                return;
            }
        }
    }

    // ── 2. 回放历史消息 ──
    let history = sqlx::query_as::<_, ChatMessage>(
        "SELECT * FROM (SELECT * FROM messages WHERE channel = $1 ORDER BY id DESC LIMIT 50) AS sub ORDER BY id ASC",
    )
    .bind(&channel_name)
    .fetch_all(&state.db)
    .await;

    if let Ok(msgs) = history {
        for msg in msgs {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json.into())).await.is_err() {
                    return;
                }
            }
        }
    }

    // ── 3. 主循环：三路 select ──
    loop {
        tokio::select! {
            // 分支 1：用户发送消息（不变）
            user_msg = receiver.next() => {
                match user_msg {
                    Some(Ok(msg)) => {
                        if let Message::Text(text) = msg {
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
                                    // ── 查用户资料，消息带上当前昵称和头像 ──
                                    let profile = sqlx::query!(
                                        "SELECT display_name, avatar_url FROM users WHERE id = $1",
                                        user.user_id
                                    )
                                    .fetch_optional(&state.db)
                                    .await
                                    .ok()
                                    .flatten();
                                    parsed_msg.display_name = profile.as_ref().and_then(|p| p.display_name.clone());
                                    parsed_msg.avatar_url = profile.as_ref().and_then(|p| p.avatar_url.clone());

                                    // ── 使用 RETURNING id 取回自增主键，广播时带上 id ──
                                    let db_result = sqlx::query_scalar!(
                                        "INSERT INTO messages (channel, username, content) VALUES ($1, $2, $3) RETURNING id",
                                        &parsed_msg.channel,
                                        &parsed_msg.username,
                                        &parsed_msg.content
                                    )
                                    .fetch_one(&state.db)
                                    .await;

                                    match db_result {
                                        Ok(new_id) => {
                                            parsed_msg.id = Some(new_id);
                                            parsed_msg.created_at = Some(chrono::Utc::now());
                                            let _ = tx.send(parsed_msg);
                                        }
                                        Err(e) => println!("数据库写入失败: {}", e),
                                    }
                                }
                                Err(e) => println!("解析失败: {}", e),
                            }
                        }
                    }
                    _ => break,
                }
            }

            // 分支 2：聊天消息广播（不变）
            recv_result = rx.recv() => {
                if let Ok(broadcast_msg) = recv_result {
                    if let Ok(json_text) = serde_json::to_string(&broadcast_msg) {
                        if sender.send(Message::Text(json_text.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }

            // 分支 3：控制事件广播（新增 — 消息删除等）
            ctrl_result = crx.recv() => {
                if let Ok(event) = ctrl_result {
                    if let Ok(json) = serde_json::to_string(&event) {
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }
}
