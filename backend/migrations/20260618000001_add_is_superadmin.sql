-- 超级管理员标志：仅 SuperAdmin 可执行危险操作（删除用户/频道/消息、升降管理员）
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_superadmin BOOLEAN NOT NULL DEFAULT false;
UPDATE users SET is_superadmin = true WHERE username = 'SuperAdmin';
