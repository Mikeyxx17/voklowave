//! WebSocket 实时消息处理：JWT 校验、协议升级、消息广播、历史回放、心跳保活。
//!
//! 新增：控制事件通道监听（消息删除等），墓碑表同步。

use crate::middleware::auth::{AuthUser, Claims};
use crate::models::ChatMessage;
use crate::state::{AppState, ControlEvent};
use axum::extract::{
    Path, Query, WebSocketUpgrade as Ws,
    ws::{Message, WebSocket},
};
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse};
use futures::{sink::SinkExt, stream::StreamExt};
use jsonwebtoken::{DecodingKey, Validation, decode};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

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

    let secret_string = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET 环境变量未设置");

    let token_data = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret_string.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data,
        Err(err) => {
            warn!("WebSocket 令牌验票失败: {}", err);
            return (StatusCode::UNAUTHORIZED, "认证令牌无效或已过期").into_response();
        }
    };

    // 校验 token_version：改密码后旧 WebSocket 连接自动失效
    let current_version = sqlx::query_scalar!(
        "SELECT token_version FROM users WHERE id = $1",
        token_data.claims.sub
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match current_version {
        None => return (StatusCode::UNAUTHORIZED, "用户不存在").into_response(),
        Some(db_version) if db_version != token_data.claims.token_version => {
            warn!(
                user_id = token_data.claims.sub,
                token_ver = token_data.claims.token_version,
                db_ver = db_version,
                "WS Token 版本不匹配，令牌已失效"
            );
            return (StatusCode::UNAUTHORIZED, "认证令牌已失效，请重新登录").into_response();
        }
        _ => {}
    }

    // 校验会话是否仍然有效（被踢出则拒绝 WebSocket 连接）
    let session_active = sqlx::query_scalar!(
        "SELECT is_active FROM sessions WHERE jti = $1",
        token_data.claims.jti
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    if session_active == Some(false) {
        warn!(
            user_id = token_data.claims.sub,
            jti = %token_data.claims.jti,
            "WS 会话已被踢出"
        );
        return (StatusCode::UNAUTHORIZED, "会话已被终止").into_response();
    }

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
    let global_tx = state.global_events.clone();  // 新增：全局用户事件

    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();
    let mut crx = control_tx.subscribe();  // 新增：控制事件订阅
    let mut grx = global_tx.subscribe();   // 新增：全局事件订阅

    info!(
        user = %user.username,
        channel = %channel_name,
        "WebSocket 连接建立"
    );

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

    // ── 2.5 下发已有表情回应（防止新上线客户端看不到历史反应） ──
    {
        let reactions = sqlx::query!(
            "SELECT r.message_id, r.username, r.emoji \
             FROM message_reactions r \
             JOIN messages m ON m.id = r.message_id \
             WHERE m.channel = $1 \
             ORDER BY m.id DESC LIMIT 300",
            channel_name
        )
        .fetch_all(&state.db)
        .await;

        if let Ok(rows) = reactions {
            for row in rows {
                let event = serde_json::json!({
                    "type": "reaction_toggled",
                    "message_id": row.message_id,
                    "emoji": row.emoji,
                    "username": row.username,
                    "action": "added"
                });
                if sender.send(Message::Text(event.to_string().into())).await.is_err() {
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
                                            // 广播 MessageCreated 到管理后台（先提取字段再 send，避免 clone）
                                            let _ = state.admin_events.send(ControlEvent::MessageCreated {
                                                message_id: new_id,
                                                channel: channel_name.clone(),
                                                username: user.username.clone(),
                                            });
                                            let _ = tx.send(parsed_msg);
                                        }
                                        Err(e) => error!("消息写入数据库失败: {}", e),
                                    }
                                }
                                Err(e) => warn!("消息 JSON 解析失败: {}", e),
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

            // 分支 4：全局用户事件（UserDeleted 等 — 驱动前端自动登出）
            global_result = grx.recv() => {
                if let Ok(ControlEvent::UserDeleted { user_id }) = global_result {
                    if user_id == user.user_id {
                        let ev = serde_json::json!({"type":"user_deleted","user_id":user_id});
                        let _ = sender.send(Message::Text(ev.to_string().into())).await;
                        info!(user_id, username = %user.username, "用户被管理员删除，WebSocket 断开");
                        break;
                    }
                }
            }
        }
    }
}

/// WebSocket 管理后台实时推送：仅管理员可连接，订阅全局 admin_events 通道。
pub async fn admin_ws_handler(
    ws: Ws,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
) -> impl IntoResponse {
    let token = match query.token {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "缺少认证令牌").into_response(),
    };

    let secret_string = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET 环境变量未设置");

    let token_data = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret_string.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data,
        Err(err) => {
            warn!("Admin WS 令牌验票失败: {}", err);
            return (StatusCode::UNAUTHORIZED, "认证令牌无效或已过期").into_response();
        }
    };

    // 必须为管理员
    let is_admin = token_data.claims.is_admin
        || sqlx::query_scalar!("SELECT is_admin FROM users WHERE id = $1", token_data.claims.sub)
            .fetch_optional(&state.db).await.ok().flatten().unwrap_or(false);

    if !is_admin {
        return (StatusCode::FORBIDDEN, "仅管理员可连接管理后台推送").into_response();
    }

    ws.on_upgrade(|socket| handle_admin_socket(socket, state))
}

/// 管理后台 WebSocket 主循环：持续推送全局 admin_events 给前端。
async fn handle_admin_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut arx = state.admin_events.subscribe();

    info!("Admin WebSocket 连接建立");

    loop {
        tokio::select! {
            // 接收 admin 事件并推送到前端
            event = arx.recv() => {
                match event {
                    Ok(ev) => {
                        if let Ok(json) = serde_json::to_string(&ev) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(skipped = n, "Admin WS 事件积压");
                        continue;
                    }
                    Err(_) => break,
                }
            }
            // 客户端消息（主要是 ping 和关闭）
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}
