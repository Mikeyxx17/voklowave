-- 为 users 表添加 token_version 字段，用于实现改密码后旧 JWT 自动失效。
-- 每次修改密码时 token_version 递增，JWT Claims 中携带签发时的 version，
-- 校验时对比数据库中的当前 version，不一致则拒绝请求。
ALTER TABLE users ADD COLUMN token_version INTEGER NOT NULL DEFAULT 1;
