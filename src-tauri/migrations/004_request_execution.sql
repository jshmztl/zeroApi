-- 004_request_execution: RequestExecution 领域模型
-- 一次请求执行记录，独立于 Request 保存。
-- 旧 history 表的数据迁移由 Rust 迁移（migrate_history_to_executions）完成。

CREATE TABLE IF NOT EXISTS request_executions (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL,
    method TEXT NOT NULL,
    url TEXT NOT NULL,
    status_code INTEGER,
    duration_ms INTEGER,
    size_bytes INTEGER,
    content_type TEXT,
    success INTEGER NOT NULL DEFAULT 0,
    error_code TEXT,
    error_message TEXT,
    response TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(request_id) REFERENCES requests(id)
);
