-- 禁言功能：muted_until 为 NULL 表示未禁言，非 NULL 表示禁言到期时间
ALTER TABLE users ADD COLUMN IF NOT EXISTS muted_until TIMESTAMPTZ;
