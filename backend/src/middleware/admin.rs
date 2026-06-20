//! 管理员中间件：AdminUser 校验 JWT 中的 is_admin 声明（无需 DB 查询）；
//! SuperAdminUser 在此基础上额外校验 is_superadmin 字段（用于危险操作）。

use crate::middleware::auth::{AuthUser, Claims};
use crate::state::AppState;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use tracing::warn;

/// 管理员身份提取器 —— 先从 JWT Claims 读取 is_admin（零 DB 查询）。
/// 若 JWT 中 is_admin 为 false，回退查数据库（处理升级后未重新登录的情况）。
/// handler 中声明 `admin: AdminUser` 即可触发校验。
pub struct AdminUser(pub AuthUser);

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // 先校验 JWT 并构建 AuthUser
        let user = AuthUser::from_request_parts(parts, state).await?;

        // 从 JWT Claims 中读取 is_admin
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let secret_string =
            std::env::var("JWT_SECRET").expect("JWT_SECRET 环境变量未设置");

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret_string.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

        if token_data.claims.is_admin {
            return Ok(AdminUser(user));
        }

        // JWT 中 is_admin 为 false，回退查 DB（用户可能刚被升级）
        let db_is_admin = sqlx::query_scalar!(
            "SELECT is_admin FROM users WHERE id = $1",
            user.user_id
        )
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

        if db_is_admin {
            Ok(AdminUser(user))
        } else {
            warn!(
                user_id = user.user_id,
                username = %user.username,
                "非管理员尝试访问管理接口"
            );
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// 超级管理员提取器 —— 在 AdminUser 基础上额外校验 is_superadmin 字段。
/// 仅用于危险操作（删除用户/频道/消息、升降管理员）。
pub struct SuperAdminUser(pub AuthUser);

impl FromRequestParts<AppState> for SuperAdminUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let admin = AdminUser::from_request_parts(parts, state).await?;

        let is_superadmin = sqlx::query_scalar!(
            "SELECT is_superadmin FROM users WHERE id = $1",
            admin.0.user_id
        )
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

        if is_superadmin {
            Ok(SuperAdminUser(admin.0))
        } else {
            warn!(
                user_id = admin.0.user_id,
                username = %admin.0.username,
                "非超级管理员尝试访问受限接口"
            );
            Err(StatusCode::FORBIDDEN)
        }
    }
}
