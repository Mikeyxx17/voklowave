// 后端服务入口 — 启动引导、路由注册、服务器绑定

mod middleware;
mod services;
mod handlers;
mod models;
mod state;
use axum::{
    Router,
    routing::{get, post},
};
use dashmap::DashMap;
use dotenvy::dotenv;
use handlers::{
    create_channel, get_channels, get_current_user, guest_login, login, register, ws_handler,
};
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use std::env;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    dotenv().ok();

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
    tokio::spawn(services::cleanup::spawn_cleanup_task(
        pool.clone(),
        cleanup_interval,
        max_age_hours,
    ));
    let channels = Arc::new(DashMap::new());
    let state = AppState {
        db: pool.clone(),
        channels,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let saved_channels = sqlx::query!("SELECT name FROM channels")
        .fetch_all(&state.db)
        .await
        .expect("无法从数据库加载频道列表");

    for row in saved_channels {
        let (tx, _rx) = broadcast::channel(100);
        state.channels.insert(row.name, tx);
    }

    println!("已成功加载 {} 个频道", state.channels.len());

    let app = Router::new()
        .route("/ws/{channel}", get(ws_handler))
        .route("/api/channels", get(get_channels).post(create_channel))
        .route("/api/login", post(login))
        .route("/api/register", post(register))
        .route("/api/guest_login", post(guest_login))
        .route("/api/me", get(get_current_user))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("后端引擎已就绪：http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
