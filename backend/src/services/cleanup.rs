use sqlx::PgPool;
use std::time::Duration;

/// 启动后台定时任务：每隔 interval_secs 秒，删除超过 max_age_hours 小时的访客及其消息
pub async fn spawn_cleanup_task(pool: PgPool, interval_secs: u64, max_age_hours: u64) {
    println!(
        "🚀 后台游客清理任务已启动，检查间隔 {} 秒",
        interval_secs
    );

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));

        loop {
            ticker.tick().await;
            println!("🧹 开始执行游客清理...");

            // 使用事务保证消息与用户同步删除，不留孤儿数据
            let mut tx = match pool.begin().await {
                Ok(transaction) => transaction,
                Err(e) => {
                    println!("❌ 开启清理事务失败: {}", e);
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
                        println!("❌ 提交清理事务失败: {}", e);
                    } else {
                        println!(
                            "✅ 清理完成：删除 {} 个访客、{} 条消息",
                            user_rows.rows_affected(),
                            msg_rows.rows_affected()
                        );
                    }
                }
                (err_msg, err_user) => {
                    println!(
                        "❌ 清理出错，事务已回滚。消息: {:?}, 用户: {:?}",
                        err_msg, err_user
                    );
                }
            }
        }
    });
}
