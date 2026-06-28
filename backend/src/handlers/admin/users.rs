//! 用户管理：列表、删除、升降管理员。

use crate::middleware::admin::{AdminUser, SuperAdminUser};
use crate::state::{AppState, ControlEvent};
use axum::Json;
use axum::extract::{ConnectInfo, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::net::SocketAddr;
use tracing::{error, info};

use super::AdminPagination;

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
            "SELECT id, username, email, is_guest, is_verified, is_admin, is_superadmin, is_owner, muted_until, created_at \
             FROM users ORDER BY id DESC LIMIT $1 OFFSET $2",
            limit, offset
        ).fetch_all(&state.db).await.unwrap_or_default();
        let list: Vec<serde_json::Value> = rows.iter().map(|u| serde_json::json!({
            "id": u.id, "username": u.username, "email": u.email,
            "is_guest": u.is_guest, "is_verified": u.is_verified,
            "is_admin": u.is_admin, "is_superadmin": u.is_superadmin,
            "is_owner": u.is_owner,
            "muted_until": u.muted_until,
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
            "SELECT id, username, email, is_guest, is_verified, is_admin, is_superadmin, is_owner, muted_until, created_at \
             FROM users WHERE username ILIKE $1 OR email ILIKE $1 \
             ORDER BY id DESC LIMIT $2 OFFSET $3",
            pattern, limit, offset
        ).fetch_all(&state.db).await.unwrap_or_default();
        let list: Vec<serde_json::Value> = rows.iter().map(|u| serde_json::json!({
            "id": u.id, "username": u.username, "email": u.email,
            "is_guest": u.is_guest, "is_verified": u.is_verified,
            "is_admin": u.is_admin, "is_superadmin": u.is_superadmin,
            "is_owner": u.is_owner,
            "muted_until": u.muted_until,
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

    // 禁止删除超级管理员和 Owner
    let target = sqlx::query!(
        "SELECT username, email, is_superadmin, is_owner FROM users WHERE id = $1", user_id
    )
    .fetch_optional(&state.db).await;

    let (target_username, target_email) = match target {
        Ok(Some(t)) if t.is_owner => {
            return (StatusCode::FORBIDDEN, "不能删除 Owner").into_response();
        }
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

    let current = sqlx::query!("SELECT username, is_admin, is_superadmin, is_owner FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db).await;

    let (target_name, _was_admin) = match current {
        Ok(Some(ref c)) if c.is_owner => {
            return (StatusCode::FORBIDDEN, "不能修改 Owner 的权限").into_response();
        }
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

/// 禁言/解除禁言请求体
#[derive(serde::Deserialize)]
pub struct MuteInput {
    /// 禁言时长（分钟），null 或 0 表示解除禁言
    pub duration_minutes: Option<i64>,
}

/// PATCH /api/admin/users/{id}/mute — 禁言或解除禁言
pub async fn admin_mute_user(
    State(state): State<AppState>,
    admin: AdminUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(user_id): Path<i32>,
    Json(input): Json<MuteInput>,
) -> impl IntoResponse {
    if user_id == admin.0.user_id {
        return (StatusCode::BAD_REQUEST, "不能禁言自己").into_response();
    }

    // 检查目标用户是否存在 + 是否为 Owner
    let target = sqlx::query!("SELECT username, is_owner FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db).await;

    let target_name = match target {
        Ok(Some(t)) if t.is_owner => {
            return (StatusCode::FORBIDDEN, "不能禁言 Owner").into_response();
        }
        Ok(Some(t)) => t.username,
        Ok(None) => return (StatusCode::NOT_FOUND, "用户不存在").into_response(),
        Err(e) => { error!("{}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };

    match input.duration_minutes {
        Some(mins) if mins > 0 => {
            // 禁言
            if let Err(e) = sqlx::query!(
                "UPDATE users SET muted_until = NOW() + $1 * INTERVAL '1 minute' WHERE id = $2",
                mins as f64, user_id
            )
            .execute(&state.db).await
            {
                error!("禁言失败: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }

            let _ = sqlx::query!(
                "INSERT INTO admin_audit_logs (admin_id, action, target, ip_address) VALUES ($1, 'mute_user', $2, $3)",
                admin.0.user_id,
                format!("禁言 {} {}分钟", target_name, mins),
                addr.ip().to_string()
            ).execute(&state.db).await;

            info!(admin = %admin.0.username, target = %target_name, mins, "管理员禁言用户");
            StatusCode::NO_CONTENT.into_response()
        }
        _ => {
            // 解除禁言
            if let Err(e) = sqlx::query!("UPDATE users SET muted_until = NULL WHERE id = $1", user_id)
                .execute(&state.db).await
            {
                error!("解除禁言失败: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }

            let _ = sqlx::query!(
                "INSERT INTO admin_audit_logs (admin_id, action, target, ip_address) VALUES ($1, 'unmute_user', $2, $3)",
                admin.0.user_id,
                format!("解除禁言 {}", target_name),
                addr.ip().to_string()
            ).execute(&state.db).await;

            info!(admin = %admin.0.username, target = %target_name, "管理员解除禁言");
            StatusCode::NO_CONTENT.into_response()
        }
    }
}

/// 批量操作用户请求体
#[derive(serde::Deserialize)]
pub struct BatchUserInput {
    pub ids: Vec<i32>,
}

/// POST /api/admin/users/batch-delete — 批量删除用户
pub async fn admin_batch_delete_users(
    State(state): State<AppState>,
    admin: SuperAdminUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<BatchUserInput>,
) -> impl IntoResponse {
    if input.ids.is_empty() {
        return (StatusCode::BAD_REQUEST, "ids 不能为空").into_response();
    }

    let ids = input.ids;

    // 检查是否包含自己或 Owner
    for &uid in &ids {
        if uid == admin.0.user_id {
            return (StatusCode::BAD_REQUEST, "不能删除自己的账号").into_response();
        }
    }

    // 过滤掉 Owner（Owner 不可删）
    let owners = sqlx::query_scalar!(
        "SELECT id FROM users WHERE id = ANY($1) AND is_owner = TRUE",
        &ids
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let deletable: Vec<i32> = ids.into_iter().filter(|id| !owners.contains(id)).collect();

    if deletable.is_empty() {
        return (StatusCode::BAD_REQUEST, "所选用户均不可删除").into_response();
    }

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => { error!("{}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    };

    for &uid in &deletable {
        let _ = sqlx::query!("DELETE FROM message_reactions WHERE username IN (SELECT username FROM users WHERE id = $1)", uid)
            .execute(&mut *tx).await;
        let _ = sqlx::query!("DELETE FROM message_reactions WHERE message_id IN (SELECT id FROM messages WHERE username IN (SELECT username FROM users WHERE id = $1))", uid)
            .execute(&mut *tx).await;
        let _ = sqlx::query!("DELETE FROM messages WHERE username IN (SELECT username FROM users WHERE id = $1)", uid)
            .execute(&mut *tx).await;
        let _ = sqlx::query!("DELETE FROM verification_codes WHERE email IN (SELECT email FROM users WHERE id = $1)", uid)
            .execute(&mut *tx).await;
        let _ = sqlx::query!("DELETE FROM sessions WHERE user_id = $1", uid)
            .execute(&mut *tx).await;
        let _ = sqlx::query!("DELETE FROM users WHERE id = $1", uid)
            .execute(&mut *tx).await;
    }

    if let Err(e) = tx.commit().await {
        error!("批量删除用户事务失败: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    for &uid in &deletable {
        let _ = state.admin_events.send(ControlEvent::UserDeleted { user_id: uid });
        let _ = state.global_events.send(ControlEvent::UserDeleted { user_id: uid });
    }

    let _ = sqlx::query!(
        "INSERT INTO admin_audit_logs (admin_id, action, target, ip_address) VALUES ($1, 'batch_delete_users', $2, $3)",
        admin.0.user_id,
        format!("批量删除 {} 个用户", deletable.len()),
        addr.ip().to_string()
    ).execute(&state.db).await;

    info!(admin = %admin.0.username, count = deletable.len(), "管理员批量删除用户");
    Json(serde_json::json!({ "deleted": deletable.len(), "skipped_owners": owners.len() })).into_response()
}

/// POST /api/admin/users/batch-toggle-admin — 批量切换管理员身份
pub async fn admin_batch_toggle_admin(
    State(state): State<AppState>,
    admin: SuperAdminUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<BatchUserInput>,
) -> impl IntoResponse {
    if input.ids.is_empty() {
        return (StatusCode::BAD_REQUEST, "ids 不能为空").into_response();
    }

    let ids = input.ids;

    // 过滤掉自己、Owner、SuperAdmin
    let protected: Vec<i32> = sqlx::query_scalar!(
        "SELECT id FROM users WHERE id = ANY($1) AND (id = $2 OR is_owner = TRUE OR is_superadmin = TRUE)",
        &ids, admin.0.user_id
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let togglable: Vec<i32> = ids.into_iter().filter(|id| !protected.contains(id)).collect();

    if togglable.is_empty() {
        return (StatusCode::BAD_REQUEST, "所选用户均不可修改").into_response();
    }

    for &uid in &togglable {
        let _ = sqlx::query!(
            "UPDATE users SET is_admin = NOT is_admin, \
             token_version = token_version + CASE WHEN is_admin THEN 1 ELSE 0 END \
             WHERE id = $1",
            uid
        )
        .execute(&state.db).await;

        let _ = state.admin_events.send(ControlEvent::UserAdminToggled { user_id: uid });
    }

    let _ = sqlx::query!(
        "INSERT INTO admin_audit_logs (admin_id, action, target, ip_address) VALUES ($1, 'batch_toggle_admin', $2, $3)",
        admin.0.user_id,
        format!("批量升降 {} 个用户", togglable.len()),
        addr.ip().to_string()
    ).execute(&state.db).await;

    info!(admin = %admin.0.username, count = togglable.len(), skipped = protected.len(), "管理员批量升降用户");
    Json(serde_json::json!({ "toggled": togglable.len(), "skipped": protected.len() })).into_response()
}
