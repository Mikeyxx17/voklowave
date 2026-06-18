-- 为墓碑表添加 UNIQUE 约束，防止因代码缺陷产生重复的删除记录。
ALTER TABLE deleted_messages ADD CONSTRAINT deleted_messages_id_unique UNIQUE (id);
