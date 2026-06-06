-- Add migration script here
CREATE TABLE messages (
    id         SERIAL PRIMARY KEY,                  -- 自增整数主键
    channel    TEXT NOT NULL,                       -- 频道名，不允许为空
    username   TEXT NOT NULL,                       -- 用户名，不允许为空
    content    TEXT NOT NULL,                       -- 消息内容，不允许为空
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()   -- 带时区的时间戳，默认为插入时的时间
);
