//! 管理员中间件：校验当前用户 is_admin 标志，拒绝非管理员访问。

use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use tracing::warn;

/// 管理员身份提取器 —— handler 中声明 `admin: AdminUser` 即可触发校验。
/// 内部先通过 AuthUser 校验 JWT，再查询 is_admin 字段。
pub struct AdminUser(pub AuthUser);

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;

        let is_admin = sqlx::query_scalar!(
            "SELECT is_admin FROM users WHERE id = $1",
            user.user_id
        )
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

        if is_admin {
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
