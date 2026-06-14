-- 表情回应表：记录用户对消息的表情反应。
-- UNIQUE(message_id, username, emoji) 保证同一人对同一条消息同个表情只能点一次，
-- 同时用作 toggle 逻辑：INSERT 成功 = 添加反应，冲突 = 已有反应 → DELETE 移除。
-- ON DELETE CASCADE：消息被硬删除时，关联的反应自动清理。
CREATE TABLE message_reactions (
    id         SERIAL PRIMARY KEY,
    message_id INTEGER NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    username   TEXT NOT NULL,
    emoji      TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(message_id, username, emoji)
);
