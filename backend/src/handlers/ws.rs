// WebSocket 聊天消息处理器 — 连接升级、生命周期管理、消息广播

use crate::models::ChatMessage;
use crate::state::AppState;
use axum::extract::{
    Path, WebSocketUpgrade as Ws,
    ws::{Message, WebSocket},
};
use axum::{extract::State, response::IntoResponse};
use futures::{sink::SinkExt, stream::StreamExt};

pub async fn ws_handler(
    ws: Ws,
    Path(channel_name): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state, channel_name))
}

async fn handle_socket(socket: WebSocket, state: AppState, channel_name: String) {
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
