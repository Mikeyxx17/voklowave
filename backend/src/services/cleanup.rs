use sqlx::PgPool;
use std::time::Duration;

pub async fn spawn_cleanup_task(pool: PgPool, interval_secs: u64, max_age_hours: u64) {
    // 打印日志，证明后台清洁工成功上岗
    println!(
        "🚀 后台游客清理任务已成功挂载，每 {} 秒检查一次...",
        interval_secs
    );

    tokio::spawn(async move {
        // 创建定时器
        let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));

        loop {
            // 等待倒计时
            ticker.tick().await;
            println!("🧹 清洁工开始巡逻...");

            // 1. 开启数据库事务，确保"同生共死"的原子性
            let mut tx = match pool.begin().await {
                Ok(transaction) => transaction,
                Err(e) => {
                    println!("❌ 开启清理事务失败: {}", e);
                    continue; // 这一次失败了不要紧，跳过并等待下一次循环
                }
            };

            // 1. 第一步：删消息
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

            // 2. 第二步：删账号
            let user_result = sqlx::query(
                "DELETE FROM users \
                WHERE is_guest = true \
                AND created_at < NOW() - ($1 || ' hours')::INTERVAL"
            )
            .bind(max_hours)
            .execute(&mut *tx)
            .await;

            // 4. 检查两步执行结果，决定提交还是回滚
            match (msg_result, user_result) {
                (Ok(msg_rows), Ok(user_rows)) => {
                    // 两步都成功了！按下 COMMIT 按钮让修改生效
                    if let Err(e) = tx.commit().await {
                        println!("❌ 提交清理事务失败: {}", e);
                    } else {
                        println!(
                            "✅ 清理完成！成功驱逐了 {} 个过期游客，并回收了他们发的 {} 条消息。",
                            user_rows.rows_affected(),
                            msg_rows.rows_affected()
                        );
                    }
                }
                (err_msg, err_user) => {
                    // 只要有任何一步失败了，整个事务自动回滚（Rollback），绝对不留孤儿数据
                    println!(
                        "❌ 清理过程中发生错误。消息删除结果: {:?}, 用户删除结果: {:?}",
                        err_msg, err_user
                    );
                    // tx 会在离开作用域时自动执行 Rollback，我们只需要记录日志即可
                }
            }
        }
    });
}
