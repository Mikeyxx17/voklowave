-- Add migration script here
-- 为 users 表一次性追加邮箱验证相关的三个字段
ALTER TABLE users 
    ADD COLUMN is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN email_verify_token TEXT,
    ADD COLUMN token_expires_at TIMESTAMPTZ;