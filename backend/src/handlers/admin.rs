//! 管理员 API：仪表盘、用户管理、频道管理、消息审计、操作日志。
//!
//! 所有读接口通过 AdminUser（JWT is_admin 声明）校验；
//! 危险写操作通过 SuperAdminUser（is_superadmin 字段）校验。

use crate::middleware::admin::{AdminUser, SuperAdminUser};
use crate::state::{AppState, ControlEvent};
use axum::Json;
use axum::extract::{ConnectInfo, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use std::net::SocketAddr;
use tracing::{error, info};

// ═══════════════════════════════════════════════════════════════════════════
// 通用分页参数
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct AdminPagination {
    #[serde(default)]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub limit: i64,
    #[serde(default)]
    pub q: String,
}

fn default_page_size() -> i64 { 20 }

// ═══════════════════════════════════════════════════════════════════════════
// 仪表盘
// ═══════════════════════════════════════════════════════════════════════════

/// GET /api/admin/dashboard — 全站概览统计
pub async fn admin_dashboard(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> impl IntoResponse {
    let total_users = sqlx::query_scalar!(
        "SELECT COUNT(*)::int8 as \"count!\" FROM users"
    ).fetch_one(&state.db).await.ok().unwrap_or(0);
    let total_messages = sqlx::query_scalar!(
        "SELECT COUNT(*)::int8 as \"count!\" FROM messages"
    ).fetch_one(&state.db).await.ok().unwrap_or(0);
    let total_channels = sqlx::query_scalar!(
        "SELECT COUNT(*)::int8 as \"count!\" FROM channels"
    ).fetch_one(&state.db).await.ok().unwrap_or(0);
    let today_messages = sqlx::query_scalar!(
        "SELECT COUNT(*)::int8 as \"count!\" FROM messages WHERE created_at > NOW() - INTERVAL '24 hours'"
    ).fetch_one(&state.db).await.ok().unwrap_or(0);
    let guest_count = sqlx::query_scalar!(
        "SELECT COUNT(*)::int8 as \"count!\" FROM users WHERE is_guest = true"
    ).fetch_one(&state.db).await.ok().unwrap_or(0);

    Json(serde_json::json!({
        "total_users": total_users,
        "total_messages": total_messages,
        "total_channels": total_channels,
        "today_messages": today_messages,
        "guest_count": guest_count,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// 用户管理
// ═══════════════════════════════════════════════════════════════════════════

/// GET /api/admin/users — 用户列表（支持搜索、分页）
pub async fn admin_list_users(
    State(state): State<AppState>,
    _admin: AdminUser,
    Query(p): Query<AdminPagination>,
) -> impl IntoResponse {
    let page = p.page.max(0);
    let limit = p.limit.min(50).max(1);
    let offset = page * limit;

    let (list, total) = if p.q.is_empty() {
        let total = sqlx::query_scalar!("SELECT COUNT(*)::int8 as \"count!\" FROM users")
            .fetch_one(&state.db).await.ok().unwrap_or(0);
        let rows = sqlx::query!(
            "SELECT id, username, email, is_guest, is_verified, is_admin, is_superadmin, created_at \
             FROM users ORDER BY id DESC LIMIT $1 OFFSET $2",
            limit, offset
        ).fetch_all(&state.db).await.unwrap_or_default();
        let list: Vec<serde_json::Value> = rows.iter().map(|u| serde_json::json!({
            "id": u.id, "username": u.username, "email": u.email,
            "is_guest": u.is_guest, "is_verified": u.is_verified,
            "is_admin": u.is_admin, "is_superadmin": u.is_superadmin,
            "created_at": u.created_at,
        })).collect();
        (list, total)
    } else {
        let pattern = format!("%{}%", p.q);
        let total = sqlx::query_scalar!(
            "SELECT COUNT(*)::int8 as \"count!\" FROM users WHERE username ILIKE $1 OR email ILIKE $1",
            pattern
        ).fetch_one(&state.db).await.ok().unwrap_or(0);
        let rows = sqlx::query!(
            "SELECT id, username, email, is_guest, is_verified, is_admin, is_superadmin, created_at \
             FROM users WHERE username ILIKE $1 OR email ILIKE $1 \
             ORDER BY id DESC LIMIT $2 OFFSET $3",
            pattern, limit, offset
        ).fetch_all(&state.db).await.unwrap_or_default();
        let list: Vec<serde_json::Value> = rows.iter().map(|u| serde_json::json!({
            "id": u.id, "username": u.username, "email": u.email,
            "is_guest": u.is_guest, "is_verified": u.is_verified,
            "is_admin": u.is_admin, "is_superadmin": u.is_superadmin,
            "created_at": u.created_at,
        })).collect();
        (list, total)
    };

    Json(serde_json::json!({ "users": list, "total": total }))
}

/// DELETE /api/admin/users/{id} — 删除用户及其关联数据（仅超级管理员）
pub async fn admin_delete_user(
    State(state): State<AppState>,
    admin: SuperAdminUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    if user_id == admin.0.user_id {
        return (StatusCode::BAD_REQUEST, "不能删除自己的账号").into_response();
    }

    // 禁止删除超级管理员（通过 is_superadmin 字段判断，不再字符串匹配）
    let target = sqlx::query!(
        "SELECT username, email, is_superadmin FROM users WHERE id = $1", user_id
    )
    .fetch_optional(&state.db).await;

    let (target_username, target_email) = match target {
        Ok(Some(t)) if t.is_superadmin => {
            return (StatusCode::FORBIDDEN, "不能删除超级管理员").into_response();
        }
        Ok(Some(t)) => (t.username, t.email),
        Ok(None) => return (StatusCode::NOT_FOUND, "用户不存在").into_response(),
        Err(e) => { error!("{}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };

    // 收集将被删除的消息 ID 和频道（事务提交后用于广播通知）
    let messages_to_delete = sqlx::query!(
        "SELECT id, channel FROM messages WHERE username = $1",
        target_username
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => { error!("事务失败: {}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };

    // 1. 删除该用户在其他消息上留下的表情回应（修复 P1 脏数据）
    let _ = sqlx::query!(
        "DELETE FROM message_reactions WHERE username = $1",
        target_username
    )
    .execute(&mut *tx).await;
    // 2. 删除该用户自己消息上的表情回应
    let _ = sqlx::query!(
        "DELETE FROM message_reactions WHERE message_id IN (SELECT id FROM messages WHERE username = $1)",
        target_username
    )
    .execute(&mut *tx).await;
    // 3. 删除消息
    let _ = sqlx::query!(
        "DELETE FROM messages WHERE username = $1",
        target_username
    )
    .execute(&mut *tx).await;
    // 4. 删除验证码记录
    let _ = sqlx::query!(
        "DELETE FROM verification_codes WHERE email = $1",
        target_email
    )
    .execute(&mut *tx).await;
    // 5. 删除会话
    let _ = sqlx::query!("DELETE FROM sessions WHERE user_id = $1", user_id)
        .execute(&mut *tx).await;
    // 6. 删除用户
    let _ = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&mut *tx).await;

    if let Err(e) = tx.commit().await {
        error!("提交失败: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // 事务提交后广播删除事件，通知在线客户端（修复 P0 不一致）
    for msg in &messages_to_delete {
        let control_tx = state.get_or_create_control_channel(&msg.channel);
        let _ = control_tx.send(ControlEvent::MessageDeleted { message_id: msg.id });
    }
    // 通知管理后台实时刷新
    let _ = state.admin_events.send(ControlEvent::UserDeleted { user_id });
    // 通知被删用户的前端自动登出（全局事件通道）
    let _ = state.global_events.send(ControlEvent::UserDeleted { user_id });

    // 审计日志带 IP（修复 P1 追溯）
    let _ = sqlx::query!(
        "INSERT INTO admin_audit_logs (admin_id, action, target, ip_address) VALUES ($1, 'delete_user', $2, $3)",
        admin.0.user_id,
        format!("user:{}", user_id),
        addr.ip().to_string()
    ).execute(&state.db).await;

    info!(admin = %admin.0.username, target_id = user_id, ip = %addr.ip(), "管理员删除用户");
    StatusCode::NO_CONTENT.into_response()
}

/// PATCH /api/admin/users/{id}/toggle-admin — 切换管理员身份（仅超级管理员）
pub async fn admin_toggle_admin(
    State(state): State<AppState>,
    admin: SuperAdminUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    if user_id == admin.0.user_id {
        return (StatusCode::BAD_REQUEST, "不能修改自己的管理员状态").into_response();
    }

    // 超级管理员身份已由 SuperAdminUser 中间件保证，不再用字符串匹配

    let current = sqlx::query!("SELECT username, is_admin, is_superadmin FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db).await;

    let (target_name, _was_admin) = match current {
        Ok(Some(ref c)) if c.is_superadmin => {
            return (StatusCode::FORBIDDEN, "不能修改超级管理员的状态").into_response();
        }
        Ok(Some(c)) => (c.username, c.is_admin),
        Ok(None) => return (StatusCode::NOT_FOUND, "用户不存在").into_response(),
        Err(e) => { error!("toggle_admin: {}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };

    // 切换 is_admin；仅在降级时递增 token_version（强制重登拿到 is_admin: false 的 JWT）
    // 升级时 AdminUser 中间件的 DB fallback 会即时生效，无需重登
    let row = sqlx::query!(
        "UPDATE users SET is_admin = NOT is_admin, \
         token_version = token_version + CASE WHEN is_admin THEN 1 ELSE 0 END \
         WHERE id = $1 RETURNING is_admin",
        user_id
    ).fetch_optional(&state.db).await;

    match row {
        Ok(Some(r)) => {
            let desc = if r.is_admin {
                format!("将 {} 升级为管理员", target_name)
            } else {
                format!("将 {} 降级为普通用户", target_name)
            };
            let _ = sqlx::query!(
                "INSERT INTO admin_audit_logs (admin_id, action, target, ip_address) VALUES ($1, 'toggle_admin', $2, $3)",
                admin.0.user_id,
                desc,
                addr.ip().to_string()
            ).execute(&state.db).await;
            // 通知管理后台实时刷新
            let _ = state.admin_events.send(ControlEvent::UserAdminToggled { user_id });
            Json(serde_json::json!({ "is_admin": r.is_admin })).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "用户不存在").into_response(),
        Err(e) => { error!("toggle_admin: {}", e); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 频道管理
// ═══════════════════════════════════════════════════════════════════════════

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

    let _ = sqlx::query!(
        "DELETE FROM message_reactions WHERE message_id IN (SELECT id FROM messages WHERE channel = $1)", name
    ).execute(&mut *tx).await;
    let _ = sqlx::query!("DELETE FROM messages WHERE channel = $1", name)
        .execute(&mut *tx).await;
    // 清理该频道的墓碑记录
    let _ = sqlx::query!("DELETE FROM deleted_messages WHERE channel = $1", name)
        .execute(&mut *tx).await;
    let _ = sqlx::query!("DELETE FROM channels WHERE id = $1", channel_id)
        .execute(&mut *tx).await;

    if let Err(e) = tx.commit().await {
        error!("{}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // 事务提交后广播删除事件（修复 P0 不一致）—— 必须先广播再移除频道
    let control_tx = state.get_or_create_control_channel(&name);
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

// ═══════════════════════════════════════════════════════════════════════════
// 消息审计
// ═══════════════════════════════════════════════════════════════════════════

/// GET /api/admin/messages — 全局消息搜索（不受频道限制）
pub async fn admin_audit_messages(
    State(state): State<AppState>,
    _admin: AdminUser,
    Query(p): Query<AdminPagination>,
) -> impl IntoResponse {
    let limit = p.limit.min(50).max(1);
    let offset = (p.page.max(0)) * limit;

    let (list, total) = if p.q.is_empty() {
        let total = sqlx::query_scalar!("SELECT COUNT(*)::int8 as \"count!\" FROM messages")
            .fetch_one(&state.db).await.ok().unwrap_or(0);
        let rows = sqlx::query!(
            "SELECT id, channel, username, content, created_at FROM messages ORDER BY id DESC LIMIT $1 OFFSET $2",
            limit, offset
        ).fetch_all(&state.db).await.unwrap_or_default();
        let list: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
            "id": r.id, "channel": r.channel, "username": r.username,
            "content": r.content, "created_at": r.created_at,
        })).collect();
        (list, total)
    } else {
        let pattern = format!("%{}%", p.q);
        let total = sqlx::query_scalar!("SELECT COUNT(*)::int8 as \"count!\" FROM messages WHERE content ILIKE $1", pattern)
            .fetch_one(&state.db).await.ok().unwrap_or(0);
        let rows = sqlx::query!(
            "SELECT id, channel, username, content, created_at FROM messages WHERE content ILIKE $1 ORDER BY id DESC LIMIT $2 OFFSET $3",
            pattern, limit, offset
        ).fetch_all(&state.db).await.unwrap_or_default();
        let list: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
            "id": r.id, "channel": r.channel, "username": r.username,
            "content": r.content, "created_at": r.created_at,
        })).collect();
        (list, total)
    };

    Json(serde_json::json!({ "messages": list, "total": total }))
}

/// DELETE /api/admin/messages/{id} — 管理员强制删除任意消息（内容审核用）
pub async fn admin_force_delete_message(
    State(state): State<AppState>,
    admin: AdminUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(message_id): Path<i32>,
) -> impl IntoResponse {
    let msg = sqlx::query!("SELECT id, channel FROM messages WHERE id = $1", message_id)
        .fetch_optional(&state.db).await;

    match msg {
        Ok(Some(m)) => {
            // 事务包裹：删除反应 + 删除消息 + 写墓碑（修复 P0 幽灵消息）
            let mut tx = match state.db.begin().await {
                Ok(tx) => tx,
                Err(e) => { error!("事务失败: {}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
            };

            let _ = sqlx::query!(
                "DELETE FROM message_reactions WHERE message_id = $1", message_id
            ).execute(&mut *tx).await;
            let _ = sqlx::query!(
                "DELETE FROM messages WHERE id = $1", message_id
            ).execute(&mut *tx).await;
            // 写入墓碑，防止重连用户看到幽灵消息
            let _ = sqlx::query!(
                "INSERT INTO deleted_messages (id, channel) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                message_id, m.channel
            ).execute(&mut *tx).await;

            if let Err(e) = tx.commit().await {
                error!("提交失败: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }

            let control_tx = state.get_or_create_control_channel(&m.channel);
            let _ = control_tx.send(ControlEvent::MessageDeleted { message_id });
            // 通知管理后台实时刷新
            let _ = state.admin_events.send(ControlEvent::MessageDeleted { message_id });

            let _ = sqlx::query!(
                "INSERT INTO admin_audit_logs (admin_id, action, target, ip_address) VALUES ($1, 'force_delete_message', $2, $3)",
                admin.0.user_id,
                format!("message:{}", message_id),
                addr.ip().to_string()
            ).execute(&state.db).await;

            info!(admin = %admin.0.username, msg_id = message_id, ip = %addr.ip(), "管理员强制删除消息");
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "消息不存在").into_response(),
        Err(e) => { error!("{}", e); StatusCode::INTERNAL_SERVER_ERROR.into_response() }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 操作日志
// ═══════════════════════════════════════════════════════════════════════════

/// GET /api/admin/audit-logs — 管理员操作记录
pub async fn admin_audit_logs(
    State(state): State<AppState>,
    _admin: AdminUser,
    Query(p): Query<AdminPagination>,
) -> impl IntoResponse {
    let limit = p.limit.min(50).max(1);
    let offset = (p.page.max(0)) * limit;

    let total = sqlx::query_scalar!("SELECT COUNT(*)::int8 as \"count!\" FROM admin_audit_logs")
        .fetch_one(&state.db).await.ok().unwrap_or(0);

    let rows = sqlx::query!(
        "SELECT a.id, a.admin_id, u.username as admin_name, a.action, a.target, a.ip_address, a.created_at \
         FROM admin_audit_logs a JOIN users u ON a.admin_id = u.id \
         ORDER BY a.id DESC LIMIT $1 OFFSET $2",
        limit, offset
    ).fetch_all(&state.db).await.unwrap_or_default();

    let list: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.id, "admin_name": r.admin_name,
        "action": r.action, "target": r.target,
        "ip_address": r.ip_address, "created_at": r.created_at,
    })).collect();

    Json(serde_json::json!({ "logs": list, "total": total }))
}
