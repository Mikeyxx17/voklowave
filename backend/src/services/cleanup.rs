//! 后台服务任务：访客清理 + 墓碑清理 + WebSocket 广播通知。

use crate::state::ControlEvent;  // 新增：控制事件类型
use dashmap::DashMap;            // 新增
use sqlx::PgPool;
use std::sync::Arc;              // 新增
use std::time::Duration;
use tokio::sync::broadcast;      // 新增
use tracing::{debug, error, info};

/// 启动后台清理任务，定期清理过期访客和过期墓碑记录。
///
/// **访客清理**（三步走）：
///   1. 事务外先查询将要被删除的消息 ID 和频道
///   2. 开启事务删除消息和用户，提交
///   3. 提交成功后，通过 control_channels 广播删除事件，通知在线客户端即时移除
///
/// **墓碑清理**：删除超过 1 小时的墓碑记录。
pub async fn spawn_cleanup_task(
    pool: PgPool,
    control_channels: Arc<DashMap<String, broadcast::Sender<ControlEvent>>>,  // 新增：控制事件广播通道
    interval_secs: u64,
    max_age_hours: u64,
) {
    info!(
        interval_secs,
        "后台清理任务已启动"
    );

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));

        loop {
            ticker.tick().await;
            info!("开始执行清理...");

            // ── 步骤 1：事务外先查出将被删除的消息（不锁定，不阻塞） ──
            let doomed: Vec<(i32, String)> = sqlx::query_as(
                "SELECT m.id, m.channel \
                 FROM messages m \
                 JOIN users u ON m.username = u.username \
                 WHERE u.is_guest = true \
                 AND u.created_at < NOW() - ($1 || ' hours')::INTERVAL"
            )
            .bind(max_age_hours as i64)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            if doomed.is_empty() {
                debug!("清理任务：无需清理的访客");
                cleanup_tombstones(&pool).await;
                continue;
            }

            // ── 步骤 2：开启事务，执行物理删除 ──
            let mut tx = match pool.begin().await {
                Ok(transaction) => transaction,
                Err(e) => {
                    error!("开启清理事务失败: {}", e);
                    cleanup_tombstones(&pool).await;
                    continue;
                }
            };

            let max_hours = max_age_hours as i64;

            let msg_result = sqlx::query(
                "DELETE FROM messages USING users \
                WHERE messages.username = users.username \
                AND users.is_guest = true \
                AND users.created_at < NOW() - ($1 || ' hours')::INTERVAL"
            )
            .bind(max_hours)
            .execute(&mut *tx)
            .await;

            let user_result = sqlx::query(
                "DELETE FROM users \
                WHERE is_guest = true \
                AND created_at < NOW() - ($1 || ' hours')::INTERVAL"
            )
            .bind(max_hours)
            .execute(&mut *tx)
            .await;

            match (msg_result, user_result) {
                (Ok(msg_rows), Ok(user_rows)) => {
                    if let Err(e) = tx.commit().await {
                        error!("提交清理事务失败: {}", e);
                    } else {
                        info!(
                            guests = %user_rows.rows_affected(),
                            messages = %msg_rows.rows_affected(),
                            "访客清理完成"
                        );

                        // ── 步骤 3：事务提交成功后，广播删除事件给在线客户端 ──
                        // 必须在事务外广播 —— 若事务回滚，消息还在，不该通知客户端
                        for (msg_id, channel) in &doomed {
                            if let Some(tx) = control_channels.get(channel) {
                                let _ = tx.send(ControlEvent::MessageDeleted {
                                    message_id: *msg_id,
                                });
                            }
                        }
                        if !doomed.is_empty() {
                            info!(
                                count = doomed.len(),
                                "已向在线客户端广播访客消息删除事件"
                            );
                        }
                    }
                }
                (err_msg, err_user) => {
                    error!(
                        ?err_msg,
                        ?err_user,
                        "访客清理出错，事务已回滚"
                    );
                }
            }

            // ── 墓碑清理（独立执行，不受访客清理成败影响） ──
            cleanup_tombstones(&pool).await;
        }
    });
}

/// 删除超过 1 小时的墓碑记录。
/// 墓碑仅用于重连客户端同步删除事件，离线超过 1 小时的客户端重连时
/// 回放的 50 条历史消息已不包含被删消息，墓碑无保留价值。
async fn cleanup_tombstones(pool: &PgPool) {
    match sqlx::query(
        "DELETE FROM deleted_messages WHERE deleted_at < NOW() - INTERVAL '1 hour'",
    )
    .execute(pool)
    .await
    {
        Ok(result) => {
            let rows = result.rows_affected();
            if rows > 0 {
                info!(rows, "墓碑清理：删除了过期记录");
            }
        }
        Err(e) => error!("墓碑清理失败: {}", e),
    }
}
