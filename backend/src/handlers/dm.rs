//! 私聊：发起私聊、私聊列表、历史消息、WebSocket 实时通信。

use crate::middleware::auth::AuthUser;
use crate::models::dm::{DmMessage, StartDmInput};
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, Query, State, WebSocketUpgrade as Ws};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tracing::{error, info, warn};

use crate::middleware::auth::Claims;
use axum::extract::ws::{Message, WebSocket};
use jsonwebtoken::{DecodingKey, Validation, decode};
use tokio::sync::broadcast;

/// POST /api/dm/start — 发起私聊（已有返回已有，没有就新建）
pub async fn dm_start(
    State(state): State<AppState>,
    user: AuthUser,
    Json(input): Json<StartDmInput>,
) -> impl IntoResponse {
    if input.user_id == user.user_id {
        return (StatusCode::BAD_REQUEST, "不能与自己私聊").into_response();
    }

    // 检查对方是否存在且非访客
    let other = sqlx::query!("SELECT id, username FROM users WHERE id = $1 AND is_guest = FALSE", input.user_id)
        .fetch_optional(&state.db).await;

    let _other_name = match other {
        Ok(Some(ref u)) => &u.username,
        Ok(None) => return (StatusCode::NOT_FOUND, "用户不存在").into_response(),
        Err(e) => { error!("{}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };

    // 查找已有会话
    let existing = sqlx::query_scalar!(
        "SELECT p1.conversation_id FROM dm_participants p1 \
         JOIN dm_participants p2 ON p1.conversation_id = p2.conversation_id \
         WHERE p1.user_id = $1 AND p2.user_id = $2",
        user.user_id, input.user_id
    )
    .fetch_optional(&state.db).await;

    match existing {
        Ok(Some(conv_id)) => {
            Json(serde_json::json!({ "conversation_id": conv_id })).into_response()
        }
        Ok(None) => {
            // 新建会话
            let mut tx = match state.db.begin().await {
                Ok(tx) => tx,
                Err(e) => { error!("{}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
            };

            let conv_id: i32 = match sqlx::query_scalar!("INSERT INTO dm_conversations DEFAULT VALUES RETURNING id")
                .fetch_one(&mut *tx).await
            {
                Ok(id) => id,
                Err(e) => { error!("{}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
            };

            for &uid in &[user.user_id, input.user_id] {
                if let Err(e) = sqlx::query!(
                    "INSERT INTO dm_participants (conversation_id, user_id) VALUES ($1, $2)", conv_id, uid
                ).execute(&mut *tx).await {
                    error!("{}", e);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }

            if let Err(e) = tx.commit().await {
                error!("{}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }

            info!(user = %user.username, target = input.user_id, conv = conv_id, "创建私聊会话");
            Json(serde_json::json!({ "conversation_id": conv_id })).into_response()
        }
        Err(e) => { error!("{}", e); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

/// GET /api/dm/list — 我的私聊列表
pub async fn dm_list(
    State(state): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    let rows = sqlx::query!(
        "SELECT \
            c.id as conv_id, \
            p2.user_id as other_id, \
            u.username as other_username, \
            u.display_name as other_display_name, \
            u.avatar_url as other_avatar_url, \
            (SELECT content FROM dm_messages WHERE conversation_id = c.id ORDER BY created_at DESC LIMIT 1) as last_msg, \
            (SELECT created_at FROM dm_messages WHERE conversation_id = c.id ORDER BY created_at DESC LIMIT 1) as last_at \
         FROM dm_conversations c \
         JOIN dm_participants p1 ON c.id = p1.conversation_id \
         JOIN dm_participants p2 ON c.id = p2.conversation_id AND p2.user_id != $1 \
         JOIN users u ON p2.user_id = u.id \
         WHERE p1.user_id = $1 \
         ORDER BY last_at DESC NULLS LAST",
        user.user_id
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let list: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "conversation_id": r.conv_id,
        "other_user_id": r.other_id,
        "other_username": r.other_username,
        "other_display_name": r.other_display_name,
        "other_avatar_url": r.other_avatar_url,
        "last_message": r.last_msg,
        "last_message_at": r.last_at,
    })).collect();

    Json(serde_json::json!({ "conversations": list }))
}

/// 分页参数
#[derive(Deserialize)]
pub struct DmMessagesQuery {
    #[serde(default)]
    pub before: Option<i32>,
    #[serde(default = "dm_default_limit")]
    pub limit: i64,
}

fn dm_default_limit() -> i64 { 50 }

/// GET /api/dm/{id}/messages — 私聊历史消息
pub async fn dm_messages(
    State(state): State<AppState>,
    user: AuthUser,
    Path(conv_id): Path<i32>,
    Query(q): Query<DmMessagesQuery>,
) -> impl IntoResponse {
    // 校验参与者身份
    let is_member = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM dm_participants WHERE conversation_id = $1 AND user_id = $2) as \"exists!\"",
        conv_id, user.user_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(false);

    if !is_member {
        return (StatusCode::FORBIDDEN, "无权访问此私聊").into_response();
    }

    let limit = q.limit.min(50).max(1);
    let before_id = q.before.unwrap_or(i32::MAX);

    let rows = sqlx::query!(
        "SELECT m.id, m.conversation_id, m.sender_id, u.username as sender_username, m.content, m.created_at \
         FROM dm_messages m JOIN users u ON m.sender_id = u.id \
         WHERE m.conversation_id = $1 AND m.id < $2 \
         ORDER BY m.created_at DESC LIMIT $3",
        conv_id, before_id, limit
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let messages: Vec<serde_json::Value> = rows.iter().rev().map(|r| serde_json::json!({
        "id": r.id,
        "conversation_id": r.conversation_id,
        "sender_id": r.sender_id,
        "sender_username": r.sender_username,
        "content": r.content,
        "created_at": r.created_at,
    })).collect();

    Json(serde_json::json!({ "messages": messages })).into_response()
}

// ═══════════════════════════════════════════════════════════════════════════
// WebSocket
// ═══════════════════════════════════════════════════════════════════════════

/// WebSocket 连接查询参数
#[derive(Deserialize)]
pub struct DmWsQuery {
    pub token: Option<String>,
}

/// WebSocket 入口：私聊实时通信
pub async fn dm_ws_handler(
    ws: Ws,
    Path(conv_id): Path<i32>,
    State(state): State<AppState>,
    Query(query): Query<DmWsQuery>,
) -> impl IntoResponse {
    let token = match query.token {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "缺少认证令牌").into_response(),
    };

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET 未设置");

    let token_data = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(d) => d,
        Err(e) => { warn!("DM WS 令牌无效: {}", e); return (StatusCode::UNAUTHORIZED, "令牌无效").into_response(); }
    };

    let user_id = token_data.claims.sub;

    // 校验参与者身份
    let is_member = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM dm_participants WHERE conversation_id = $1 AND user_id = $2) as \"exists!\"",
        conv_id, user_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(false);

    if !is_member {
        return (StatusCode::FORBIDDEN, "无权访问此私聊").into_response();
    }

    let username = token_data.claims.username.clone();

    ws.on_upgrade(move |socket| handle_dm_socket(socket, state, conv_id, user_id, username))
}

/// 私聊 WebSocket 主循环
async fn handle_dm_socket(socket: WebSocket, state: AppState, conv_id: i32, user_id: i32, username: String) {
    let (mut sender, mut receiver) = socket.split();

    // 获取或创建该私聊的广播通道
    let tx = state.dm_channels
        .entry(conv_id)
        .or_insert_with(|| {
            let (tx, _) = broadcast::channel(100);
            tx
        })
        .clone();

    let mut rx = tx.subscribe();

    // —— 回放最近 50 条历史消息 ——
    if let Ok(rows) = sqlx::query!(
        "SELECT m.id, m.conversation_id, m.sender_id, u.username as sender_username, m.content, m.created_at \
         FROM dm_messages m JOIN users u ON m.sender_id = u.id \
         WHERE m.conversation_id = $1 ORDER BY m.created_at DESC LIMIT 50",
        conv_id
    )
    .fetch_all(&state.db)
    .await
    {
        let msgs: Vec<serde_json::Value> = rows.iter().rev().map(|r| serde_json::json!({
            "type": "dm_message",
            "id": r.id,
            "conversation_id": r.conversation_id,
            "sender_id": r.sender_id,
            "sender_username": r.sender_username,
            "content": r.content,
            "created_at": r.created_at,
        })).collect();

        for m in msgs {
            let _ = sender.send(Message::Text(m.to_string().into())).await;
        }
    }

    // —— 主循环 ——
    loop {
        tokio::select! {
            // 分支 1：用户发送消息
            user_msg = receiver.next() => {
                match user_msg {
                    Some(Ok(Message::Text(text))) => {
                        // ping/pong
                        if let Ok(ping) = serde_json::from_str::<serde_json::Value>(&text) {
                            if ping.get("type").and_then(|v| v.as_str()) == Some("ping") {
                                let _ = sender.send(Message::Text("{\"type\":\"pong\"}".into())).await;
                                continue;
                            }
                        }

                        // 解析私聊消息
                        if let Ok(mut dm_msg) = serde_json::from_str::<DmMessage>(&text) {
                            dm_msg.sender_id = user_id;
                            dm_msg.conversation_id = conv_id;

                            let ctx = dm_msg.content.trim();
                            if ctx.is_empty() || ctx.len() > 2000 {
                                continue;
                            }

                            if let Ok(row) = sqlx::query!(
                                "INSERT INTO dm_messages (conversation_id, sender_id, content) VALUES ($1, $2, $3) RETURNING id, created_at",
                                conv_id, user_id, ctx
                            )
                            .fetch_one(&state.db)
                            .await
                            {
                                let broadcast_msg = serde_json::json!({
                                    "type": "dm_message",
                                    "id": row.id,
                                    "conversation_id": conv_id,
                                    "sender_id": user_id,
                                    "sender_username": username,
                                    "content": ctx,
                                    "created_at": row.created_at,
                                });
                                let _ = tx.send(broadcast_msg);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => continue,
                }
            }

            // 分支 2：广播消息
            recv_result = rx.recv() => {
                if let Ok(msg) = recv_result {
                    if let Ok(json) = serde_json::to_string(&msg) {
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }
}
