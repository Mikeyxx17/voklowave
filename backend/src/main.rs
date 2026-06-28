//! 服务入口：加载环境变量、初始化数据库、执行迁移、注册路由并启动 HTTP 监听。
//!
//! 启动命令：`cd backend && cargo run`
//! 监听地址：`0.0.0.0:3000`

mod handlers;
mod middleware;
mod models;
mod services;
mod state;

use axum::{
    Router,
    routing::{delete, get, post},
};
use dashmap::DashMap;
use dotenvy::dotenv;
use handlers::{
    admin_ws_handler, create_channel, delete_message, dm_list, dm_messages,
    dm_start, dm_ws_handler, forgot_password,
    get_channels, get_current_user, guest_login, list_sessions, list_users, login, register,
    resend_verification, reset_password, revoke_session, search_messages, toggle_reaction,
    update_profile, verify_email, ws_handler,
};
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use std::env;
use std::net::SocketAddr; // 新增：用于 ConnectInfo
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

/// 应用主入口：依次初始化数据库、后台任务、频道缓存、CORS、路由，然后绑定端口启动。
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    // ── 启动时强制校验 JWT_SECRET，防止生产环境遗忘配置导致使用 fallback 密钥 ──
    let _ = std::env::var("JWT_SECRET")
        .expect("❌ 致命错误：未设置 JWT_SECRET 环境变量！请在 .env 中配置安全的随机密钥。");

    let database_url = env::var("DATABASE_URL").expect("请在 .env 文件中设置 DATABASE_URL");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("无法连接到数据库");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("数据库迁移失败，请检查 SQL 脚本");

    let cleanup_interval = std::env::var("CLEANUP_INTERVAL_SECS")
        .map(|s| s.parse::<u64>().unwrap_or(1800))
        .unwrap_or(1800);
    let max_age_hours = std::env::var("GUEST_MAX_AGE_HOURS")
        .map(|s| s.parse::<u64>().unwrap_or(24))
        .unwrap_or(24);

    // ── 初始化：聊天频道 + 控制事件频道 + 管理后台全局事件通道 ──
    let channels = Arc::new(DashMap::new());
    let dm_channels: Arc<DashMap<i32, tokio::sync::broadcast::Sender<serde_json::Value>>> = Arc::new(DashMap::new());
    let control_channels = Arc::new(DashMap::new());
    let (admin_events_tx, _) = broadcast::channel(256);
    let (global_events_tx, _) = broadcast::channel(256);

    let state = AppState {
        db: pool.clone(),
        channels,
        dm_channels,
        control_channels,
        admin_events: admin_events_tx,
        global_events: global_events_tx,
        login_limiter: services::rate_limit::login_limiter(),
        register_limiter: services::rate_limit::register_limiter(),
        resend_limiter: services::rate_limit::resend_limiter(),
        forgot_password_limiter: services::rate_limit::forgot_password_limiter(),
    };

    // 从数据库加载已有频道，为每个频道创建广播通道
    let saved_channels = sqlx::query!("SELECT name FROM channels")
        .fetch_all(&state.db)
        .await
        .expect("无法从数据库加载频道列表");

    for row in saved_channels {
        let (tx, _rx) = broadcast::channel(100);
        state.channels.insert(row.name.clone(), tx);
        // 新增：为已有频道创建控制事件通道
        let (ctx, _) = broadcast::channel(100);
        state.control_channels.insert(row.name, ctx);
    }

    // ── 启动后台清理任务（移到 state 构建之后，以便传入 control_channels 做删除广播） ──
    tokio::spawn(services::cleanup::spawn_cleanup_task(
        pool.clone(),
        state.control_channels.clone(), // 新增：传入控制通道，清理时通知在线客户端
        cleanup_interval,
        max_age_hours,
    ));

    info!("已成功加载 {} 个频道", state.channels.len());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/ws/{channel}", get(ws_handler))
        .route("/ws/admin", get(admin_ws_handler))
        .route("/api/channels", get(get_channels).post(create_channel))
        .route("/api/login", post(login))
        .route("/api/register", post(register))
        .route("/api/verify_email", post(verify_email))
        .route("/api/resend_verification", post(resend_verification))
        .route("/api/guest_login", post(guest_login))
        .route("/api/forgot_password", post(forgot_password))
        .route("/api/reset_password", post(reset_password))
        // ── 用户资料：GET 获取当前信息，PATCH 更新资料 ──
        .route("/api/users", get(list_users)) // 新增：@ 提及用户搜索
        .route("/api/me", get(get_current_user).patch(update_profile))
        // ── 消息删除：DELETE 硬删除自己的消息 ──
        .route("/api/messages/{id}", delete(delete_message))
        // ── 消息搜索：GET 模糊搜索历史消息 ──
        .route("/api/messages/search", get(search_messages))
        // ── 表情回应：POST 切换表情反应 ──
        .route("/api/messages/{id}/react", post(toggle_reaction))
        // ── 会话管理：列表 & 踢出 ──
        .route("/api/sessions", get(list_sessions))
        .route("/api/sessions/{id}", delete(revoke_session))
        // ── 私聊 ──
        .route("/api/dm/start", post(dm_start))
        .route("/api/dm/list", get(dm_list))
        .route("/api/dm/{id}/messages", get(dm_messages))
        .route("/ws/dm/{id}", get(dm_ws_handler))
        // ── 管理员后台 ──
        .nest("/api/admin", handlers::admin::router(state.clone()))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("后端引擎已就绪：http://{}", listener.local_addr().unwrap());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
