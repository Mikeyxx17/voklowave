-- ============================================================
-- voklowave 种子数据
-- 用法：
--   1. 生成 bcrypt 密码哈希：cargo run --bin hash -- 你的密码
--   2. 把下面占位符替换为真实值
--   3. 执行：psql -h localhost -U voklowave_user -d voklowave_database -f seed.sql
-- ============================================================

INSERT INTO users (username, email, password_hash, is_verified, is_admin, is_superadmin, is_owner, token_version)
VALUES (
    'SuperAdmin',                              -- ← 改这里：超级管理员用户名
    'micahxgenz@gmail.com',                    -- ← 改这里：用于找回密码
    '$2b$10$CJ6MYFA3H4.e1KS7YIm.9uAk4KSXERZIUz4QuWlgUIFKz7/A5FUpO',      -- ← 改这里：密码哈希
    true,
    true,
    true,
    true,
    1
)
ON CONFLICT (username) DO UPDATE
SET password_hash = EXCLUDED.password_hash,
    email         = EXCLUDED.email;
