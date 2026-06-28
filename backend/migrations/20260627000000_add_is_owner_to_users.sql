-- 添加 is_owner 字段，用于标识不可删除的 Owner 角色
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_owner BOOLEAN NOT NULL DEFAULT FALSE;
