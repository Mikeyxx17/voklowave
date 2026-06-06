-- 修改后的最终 users 表结构
CREATE TABLE users (
    id            SERIAL PRIMARY KEY,
    email         TEXT NOT NULL UNIQUE,         -- 登录凭证（保持唯一）
    password_hash TEXT NOT NULL,                -- 密码哈希
    username      TEXT NOT NULL UNIQUE,         -- 核心修改：保留并加上 UNIQUE！用于 @提及 的唯一账号名
    display_name  VARCHAR(50),                  -- 新增：用户自定义的唯美昵称（允许重复，默认 NULL）
    avatar_url    TEXT,                         -- 新增：头像的网络链接（默认 NULL）
    bio           TEXT,                         -- 新增：个性签名（默认 NULL）
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);