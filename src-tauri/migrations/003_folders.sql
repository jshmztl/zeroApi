-- 003_folders: Folder 领域模型 + V2 requests 表结构
-- folders 支持多级（parent_id 指向另一 Folder）。
-- requests 从 V1 结构演进为 V2 结构（移除 status/last_response，新增 folder_id/sort_order/query_params），
-- 旧表数据迁移与切换由 Rust 迁移（migrate_requests_v2）完成。

CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    collection_id TEXT NOT NULL,
    parent_id TEXT,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(collection_id) REFERENCES collections(id),
    FOREIGN KEY(parent_id) REFERENCES folders(id)
);

-- V2 requests 表结构
CREATE TABLE IF NOT EXISTS requests_v2 (
    id TEXT PRIMARY KEY,
    collection_id TEXT NOT NULL DEFAULT '',
    folder_id TEXT,
    name TEXT NOT NULL DEFAULT '',
    method TEXT NOT NULL DEFAULT 'GET',
    url TEXT NOT NULL DEFAULT '',
    headers TEXT NOT NULL DEFAULT '[]',
    query_params TEXT NOT NULL DEFAULT '[]',
    body TEXT,
    auth TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY(collection_id) REFERENCES collections(id),
    FOREIGN KEY(folder_id) REFERENCES folders(id)
);
