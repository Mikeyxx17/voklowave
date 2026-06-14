-- 墓碑表：记录最近被硬删除的消息 ID，防止重连客户端遗留幽灵消息。
-- 数据保留 1 小时后由后台清理任务自动删除。
CREATE TABLE deleted_messages (
    id         INTEGER NOT NULL,
    channel    TEXT NOT NULL,
    deleted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_deleted_messages_channel_time ON deleted_messages(channel, deleted_at);
