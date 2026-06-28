-- 私聊功能：会话表、参与者表、私聊消息表

-- 私聊会话（一对一对话）
CREATE TABLE dm_conversations (
    id          SERIAL PRIMARY KEY,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 参与者记录（每个会话恰好 2 人）
CREATE TABLE dm_participants (
    conversation_id  INTEGER NOT NULL REFERENCES dm_conversations(id) ON DELETE CASCADE,
    user_id          INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (conversation_id, user_id)
);

-- 私聊消息
CREATE TABLE dm_messages (
    id               SERIAL PRIMARY KEY,
    conversation_id  INTEGER NOT NULL REFERENCES dm_conversations(id) ON DELETE CASCADE,
    sender_id        INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content          TEXT NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 加速查询某用户的所有私聊会话
CREATE INDEX idx_dm_participants_user ON dm_participants(user_id);
-- 加速按会话查消息
CREATE INDEX idx_dm_messages_conversation ON dm_messages(conversation_id, created_at DESC);
