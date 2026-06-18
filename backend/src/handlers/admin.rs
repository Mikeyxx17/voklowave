//! 管理员 API：仪表盘、用户管理、频道管理、消息审计、操作日志。
//!
//! 所有接口均通过 AdminUser 提取器做权限校验。

use crate::middleware::admin::AdminUser;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
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

/// 超级管理员用户名，与 seed.sql 和前端 config.js 保持一致
fn super_admin_name() -> String {
    std::env::var("SUPER_ADMIN").unwrap_or_else(|_| "SuperAdmin".to_string())
}

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
            "SELECT id, username, email, is_guest, is_verified, is_admin, created_at \
             FROM users ORDER BY id DESC LIMIT $1 OFFSET $2",
            limit, offset
        ).fetch_all(&state.db).await.unwrap_or_default();
        let list: Vec<serde_json::Value> = rows.iter().map(|u| serde_json::json!({
            "id": u.id, "username": u.username, "email": u.email,
            "is_guest": u.is_guest, "is_verified": u.is_verified,
            "is_admin": u.is_admin, "created_at": u.created_at,
        })).collect();
        (list, total)
    } else {
        let pattern = format!("%{}%", p.q);
        let total = sqlx::query_scalar!(
            "SELECT COUNT(*)::int8 as \"count!\" FROM users WHERE username ILIKE $1 OR email ILIKE $1",
            pattern
        ).fetch_one(&state.db).await.ok().unwrap_or(0);
        let rows = sqlx::query!(
            "SELECT id, username, email, is_guest, is_verified, is_admin, created_at \
             FROM users WHERE username ILIKE $1 OR email ILIKE $1 \
             ORDER BY id DESC LIMIT $2 OFFSET $3",
            pattern, limit, offset
        ).fetch_all(&state.db).await.unwrap_or_default();
        let list: Vec<serde_json::Value> = rows.iter().map(|u| serde_json::json!({
            "id": u.id, "username": u.username, "email": u.email,
            "is_guest": u.is_guest, "is_verified": u.is_verified,
            "is_admin": u.is_admin, "created_at": u.created_at,
        })).collect();
        (list, total)
    };

    Json(serde_json::json!({ "users": list, "total": total }))
}

/// DELETE /api/admin/users/{id} — 删除用户及其关联数据
pub async fn admin_delete_user(
    State(state): State<AppState>,
    admin: AdminUser,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    if user_id == admin.0.user_id {
        return (StatusCode::BAD_REQUEST, "不能删除自己的账号").into_response();
    }

    // 任何人都不能删除超级管理员
    let target = sqlx::query!("SELECT username FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db).await;
    match target {
        Ok(Some(t)) if t.username == super_admin_name() => {
            return (StatusCode::FORBIDDEN, "不能删除超级管理员").into_response();
        }
        Ok(None) => return (StatusCode::NOT_FOUND, "用户不存在").into_response(),
        Err(e) => { error!("{}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
        _ => {}
    }

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => { error!("事务失败: {}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };

    let _ = sqlx::query!("DELETE FROM message_reactions WHERE message_id IN (SELECT id FROM messages WHERE username = (SELECT username FROM users WHERE id = $1))", user_id)
        .execute(&mut *tx).await;
    let _ = sqlx::query!("DELETE FROM messages WHERE username = (SELECT username FROM users WHERE id = $1)", user_id)
        .execute(&mut *tx).await;
    let _ = sqlx::query!("DELETE FROM sessions WHERE user_id = $1", user_id)
        .execute(&mut *tx).await;
    let _ = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&mut *tx).await;

    if let Err(e) = tx.commit().await {
        error!("提交失败: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let _ = sqlx::query!(
        "INSERT INTO admin_audit_logs (admin_id, action, target) VALUES ($1, 'delete_user', $2)",
        admin.0.user_id, format!("user:{}", user_id)
    ).execute(&state.db).await;

    info!(admin = %admin.0.username, target_id = user_id, "管理员删除用户");
    StatusCode::NO_CONTENT.into_response()
}

/// PATCH /api/admin/users/{id}/toggle-admin — 切换管理员身份
pub async fn admin_toggle_admin(
    State(state): State<AppState>,
    admin: AdminUser,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    if user_id == admin.0.user_id {
        return (StatusCode::BAD_REQUEST, "不能修改自己的管理员状态").into_response();
    }

    // 只有超级管理员才能升降管理员
    if admin.0.username != super_admin_name() {
        return (StatusCode::FORBIDDEN, "仅超级管理员可升降管理员").into_response();
    }

    // 先查询当前状态，用于日志描述
    let current = sqlx::query!("SELECT username, is_admin FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db).await;

    let target_name = match current {
        Ok(Some(c)) => c.username,
        Ok(None) => return (StatusCode::NOT_FOUND, "用户不存在").into_response(),
        Err(e) => { error!("toggle_admin: {}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };

    let row = sqlx::query!(
        "UPDATE users SET is_admin = NOT is_admin WHERE id = $1 RETURNING is_admin",
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
                "INSERT INTO admin_audit_logs (admin_id, action, target) VALUES ($1, 'toggle_admin', $2)",
                admin.0.user_id, desc
            ).execute(&state.db).await;
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

/// DELETE /api/admin/channels/{id} — 删除频道及关联消息
pub async fn admin_delete_channel(
    State(state): State<AppState>,
    admin: AdminUser,
    Path(channel_id): Path<i32>,
) -> impl IntoResponse {
    let ch = sqlx::query!("SELECT name FROM channels WHERE id = $1", channel_id)
        .fetch_optional(&state.db).await;

    let name = match ch {
        Ok(Some(c)) => c.name,
        Ok(None) => return (StatusCode::NOT_FOUND, "频道不存在").into_response(),
        Err(e) => { error!("{}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let _ = sqlx::query!("DELETE FROM message_reactions WHERE message_id IN (SELECT id FROM messages WHERE channel = $1)", name)
        .execute(&mut *tx).await;
    let _ = sqlx::query!("DELETE FROM messages WHERE channel = $1", name)
        .execute(&mut *tx).await;
    let _ = sqlx::query!("DELETE FROM channels WHERE id = $1", channel_id)
        .execute(&mut *tx).await;

    if let Err(e) = tx.commit().await {
        error!("{}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    state.channels.remove(&name);
    state.control_channels.remove(&name);

    let _ = sqlx::query!(
        "INSERT INTO admin_audit_logs (admin_id, action, target) VALUES ($1, 'delete_channel', $2)",
        admin.0.user_id, format!("channel:{}", name)
    ).execute(&state.db).await;

    info!(admin = %admin.0.username, channel = %name, "管理员删除频道");
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

/// DELETE /api/admin/messages/{id} — 管理员强制删除任意消息
pub async fn admin_force_delete_message(
    State(state): State<AppState>,
    admin: AdminUser,
    Path(message_id): Path<i32>,
) -> impl IntoResponse {
    let msg = sqlx::query!("SELECT id, channel FROM messages WHERE id = $1", message_id)
        .fetch_optional(&state.db).await;

    match msg {
        Ok(Some(m)) => {
            let _ = sqlx::query!("DELETE FROM message_reactions WHERE message_id = $1", message_id)
                .execute(&state.db).await;
            let _ = sqlx::query!("DELETE FROM messages WHERE id = $1", message_id)
                .execute(&state.db).await;

            let control_tx = state.get_or_create_control_channel(&m.channel);
            let _ = control_tx.send(crate::state::ControlEvent::MessageDeleted { message_id });

            let _ = sqlx::query!(
                "INSERT INTO admin_audit_logs (admin_id, action, target) VALUES ($1, 'force_delete_message', $2)",
                admin.0.user_id, format!("message:{}", message_id)
            ).execute(&state.db).await;

            info!(admin = %admin.0.username, msg_id = message_id, "管理员强制删除消息");
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
        "SELECT a.id, a.admin_id, u.username as admin_name, a.action, a.target, a.created_at \
         FROM admin_audit_logs a JOIN users u ON a.admin_id = u.id \
         ORDER BY a.id DESC LIMIT $1 OFFSET $2",
        limit, offset
    ).fetch_all(&state.db).await.unwrap_or_default();

    let list: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.id, "admin_name": r.admin_name,
        "action": r.action, "target": r.target, "created_at": r.created_at,
    })).collect();

    Json(serde_json::json!({ "logs": list, "total": total }))
}
