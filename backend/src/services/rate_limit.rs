//! 简易 IP 限流器：基于内存的滑动窗口算法，防止暴力登录/注册/验证码滥用。
//!
//! 每个限流器实例对应一种速率限制规则。后台每 60 秒自动清理过期记录。

use axum::http::StatusCode;
use axum::response::IntoResponse;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 限流器：为不同路由预设不同的规则。
#[derive(Clone)]
pub struct RateLimiter {
    hits: Arc<DashMap<SocketAddr, Vec<Instant>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    /// 创建一个新的限流器实例。
    /// - `max_requests`: 时间窗口内允许的最大请求数
    /// - `window_secs`: 时间窗口长度（秒）
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        let hits = Arc::new(DashMap::new());
        let hits_clone = hits.clone();
        let window = Duration::from_secs(window_secs);

        // ── 后台定时清理过期记录 ──
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(60));
            loop {
                ticker.tick().await;
                let cutoff = Instant::now() - window;
                hits_clone.retain(|_, v: &mut Vec<Instant>| {
                    v.retain(|t| *t > cutoff);
                    !v.is_empty()
                });
            }
        });

        Self { hits, max_requests, window }
    }

    /// 检查 IP 是否超限。未超限则记录并返回 `Ok(())`；超限返回 `Err(429)`。
    /// 在 handler 开头调用：`limiter.check(addr)?;`
    pub fn check(&self, ip: SocketAddr) -> Result<(), impl IntoResponse> {
        let now = Instant::now();
        let cutoff = now - self.window;

        let mut entry = self.hits.entry(ip).or_default();
        entry.retain(|t| *t > cutoff);

        if entry.len() >= self.max_requests {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                format!("请求过于频繁，请 {} 秒后再试", self.window.as_secs()),
            ));
        }

        entry.push(now);
        Ok(())
    }
}

/// 登录限流：每 IP 每分钟最多 10 次
pub fn login_limiter() -> RateLimiter {
    RateLimiter::new(10, 60)
}

/// 注册限流：每 IP 每小时最多 5 次
pub fn register_limiter() -> RateLimiter {
    RateLimiter::new(5, 3600)
}

/// 重发验证码限流：每 IP 每小时最多 5 次
pub fn resend_limiter() -> RateLimiter {
    RateLimiter::new(5, 3600)
}
