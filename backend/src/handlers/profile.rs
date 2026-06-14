//! 用户资料编辑：PATCH /api/me — 更新当前登录用户的昵称、头像、签名。

use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

/// 资料更新请求体，所有字段均为可选，仅更新提供的字段。
#[derive(serde::Deserialize)]
pub struct UpdateProfileInput {
    /// 用户自定义昵称（最多 50 个字符），传空字符串可清空
    pub display_name: Option<String>,
    /// 头像的网络链接，传空字符串可清空
    pub avatar_url: Option<String>,
    /// 个性签名，传空字符串可清空
    pub bio: Option<String>,
}

/// @ 提及搜索查询参数。
#[derive(serde::Deserialize)]
pub struct UserSearchQuery {
    pub q: Option<String>,
}

/// 搜索用户：根据关键词模糊匹配用户名和昵称，用于 @ 提及自动补全。
/// 返回最多 10 条匹配结果。
pub async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<UserSearchQuery>,
) -> impl IntoResponse {
    let keyword = query.q.as_deref().unwrap_or("").trim().to_string();
    if keyword.is_empty() {
        return (StatusCode::OK, Json(serde_json::json!([]))).into_response();
    }

    let pattern = format!("%{}%", keyword);
    let rows = sqlx::query!(
        "SELECT username, display_name FROM users WHERE username ILIKE $1 OR display_name ILIKE $1 LIMIT 10",
        pattern
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(users) => {
            let results: Vec<serde_json::Value> = users
                .into_iter()
                .map(|r| serde_json::json!({
                    "username": r.username,
                    "display_name": r.display_name,
                }))
                .collect();
            (StatusCode::OK, Json(serde_json::json!(results))).into_response()
        }
        Err(e) => {
            println!("[用户搜索] 失败: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// 更新当前登录用户的个人资料。
/// - 仅更新提供的字段，未提供的保持原值不变
/// - 返回更新后的完整用户资料用于前端状态同步
pub async fn update_profile(
    State(state): State<AppState>,
    user: AuthUser,
    Json(input): Json<UpdateProfileInput>,
) -> impl IntoResponse {
    // ── 字段长度校验 ──
    if let Some(ref dn) = input.display_name {
        if dn.len() > 50 {
            return (StatusCode::BAD_REQUEST, "昵称不能超过 50 个字符").into_response();
        }
    }
    if let Some(ref bio) = input.bio {
        if bio.len() > 500 {
            return (StatusCode::BAD_REQUEST, "签名不能超过 500 个字符").into_response();
        }
    }

    // ── 使用 COALESCE 仅更新提供的字段 ──
    let result = sqlx::query!(
        "UPDATE users \
         SET display_name = COALESCE($1, display_name), \
             avatar_url   = COALESCE($2, avatar_url), \
             bio          = COALESCE($3, bio) \
         WHERE id = $4 \
         RETURNING id, username, email, display_name, avatar_url, bio, is_guest",
        input.display_name,
        input.avatar_url,
        input.bio,
        user.user_id
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(row) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": row.id,
                "username": row.username,
                "email": row.email,
                "display_name": row.display_name,
                "avatar_url": row.avatar_url,
                "bio": row.bio,
                "is_guest": row.is_guest,
            })),
        )
            .into_response(),
        Err(e) => {
            println!("更新用户资料失败: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
