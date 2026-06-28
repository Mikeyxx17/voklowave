//! 频道管理：列表、删除。

use crate::middleware::admin::{AdminUser, SuperAdminUser};
use crate::state::{AppState, ControlEvent};
use axum::Json;
use axum::extract::{ConnectInfo, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::net::SocketAddr;
use tracing::{error, info};

/// GET /api/admin/channels — 频道列表（含消息数）
pub async fn admin_list_channels(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> impl IntoResponse {
    let rows = sqlx::query!(
        "SELECT c.id, c.name, c.created_at, \
         COALESCE((SELECT COUNT(*)::int8 FROM messages m WHERE m.channel = c.name), 0) as msg_count \
         FROM channels c ORDER BY c.id"
    ).fetch_all(&state.db).await.unwrap_or_default();

    let list: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.id, "name": r.name,
        "created_at": r.created_at, "msg_count": r.msg_count,
    })).collect();

    Json(serde_json::json!({ "channels": list }))
}

/// DELETE /api/admin/channels/{id} — 删除频道及关联数据（仅超级管理员）
pub async fn admin_delete_channel(
    State(state): State<AppState>,
    admin: SuperAdminUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(channel_id): Path<i32>,
) -> impl IntoResponse {
    let ch = sqlx::query!("SELECT name FROM channels WHERE id = $1", channel_id)
        .fetch_optional(&state.db).await;

    let name = match ch {
        Ok(Some(c)) => c.name,
        Ok(None) => return (StatusCode::NOT_FOUND, "频道不存在").into_response(),
        Err(e) => { error!("{}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };

    // 收集消息 ID（用于广播）
    let message_ids: Vec<i32> = sqlx::query!(
        "SELECT id FROM messages WHERE channel = $1",
        name
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .iter()
    .map(|r| r.id)
    .collect();

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // 按依赖顺序删除：先删 reactions（引用 messages），再删 messages，最后删频道
    if let Err(e) = sqlx::query!(
        "DELETE FROM message_reactions WHERE message_id IN (SELECT id FROM messages WHERE channel = $1)", name
    ).execute(&mut *tx).await {
        error!("删除频道 {}: 清理表情回应失败: {}", name, e);
    }
    if let Err(e) = sqlx::query!("DELETE FROM messages WHERE channel = $1", name)
        .execute(&mut *tx).await
    {
        error!("删除频道 {}: 清理消息失败: {}", name, e);
    }
    // 清理该频道的墓碑记录
    if let Err(e) = sqlx::query!("DELETE FROM deleted_messages WHERE channel = $1", name)
        .execute(&mut *tx).await
    {
        error!("删除频道 {}: 清理墓碑失败: {}", name, e);
    }
    if let Err(e) = sqlx::query!("DELETE FROM channels WHERE id = $1", channel_id)
        .execute(&mut *tx).await
    {
        error!("删除频道 {}: 删除频道记录失败: {}", name, e);
    }

    if let Err(e) = tx.commit().await {
        error!("删除频道 {}: 提交事务失败: {}", name, e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // 事务提交后广播删除事件 —— 必须先广播再移除频道
    let control_tx = state.get_or_create_control_channel(&name);
    // 先通知频道内在线用户"频道已删除"，客户端收到后自动跳走
    let _ = control_tx.send(ControlEvent::ChannelDeleted { name: name.clone() });
    // 再逐条广播消息删除事件（墓碑同步）
    for msg_id in &message_ids {
        let _ = control_tx.send(ControlEvent::MessageDeleted { message_id: *msg_id });
    }
    // 通知管理后台实时刷新
    let _ = state.admin_events.send(ControlEvent::ChannelDeleted { name: name.clone() });

    state.channels.remove(&name);
    state.control_channels.remove(&name);

    let _ = sqlx::query!(
        "INSERT INTO admin_audit_logs (admin_id, action, target, ip_address) VALUES ($1, 'delete_channel', $2, $3)",
        admin.0.user_id,
        format!("channel:{}", name),
        addr.ip().to_string()
    ).execute(&state.db).await;

    info!(admin = %admin.0.username, channel = %name, ip = %addr.ip(), "管理员删除频道");
    StatusCode::NO_CONTENT.into_response()
}
