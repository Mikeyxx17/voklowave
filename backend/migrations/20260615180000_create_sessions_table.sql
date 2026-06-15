-- 会话表：记录用户每次登录产生的 JWT 会话。
-- 用于实现活跃会话列表、远程踢出、新设备登录检测。
CREATE TABLE sessions (
    id         SERIAL PRIMARY KEY,
    user_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    jti        TEXT NOT NULL UNIQUE,
    ip_address TEXT,
    user_agent TEXT,
    is_active  BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_sessions_user_id ON sessions(user_id, is_active);
