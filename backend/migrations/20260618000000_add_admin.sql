-- 管理员字段 + 操作审计日志表
ALTER TABLE users ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE admin_audit_logs (
    id          SERIAL PRIMARY KEY,
    admin_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action      VARCHAR(100) NOT NULL,
    target      VARCHAR(255),
    ip_address  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_admin_audit_admin_id ON admin_audit_logs(admin_id);
CREATE INDEX idx_admin_audit_created_at ON admin_audit_logs(created_at);
