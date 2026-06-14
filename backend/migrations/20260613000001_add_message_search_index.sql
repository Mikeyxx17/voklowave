-- 消息搜索支持：启用 pg_trgm 扩展，创建三元组（trigram）索引加速 ILIKE 模糊搜索。
-- pg_trgm 将文本拆分为连续三字符片段建立索引，使 LIKE/ILIKE '%关键词%' 查询可以利用索引。
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX idx_messages_content_trgm ON messages USING gin (content gin_trgm_ops);
