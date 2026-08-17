-- 001_initial: V1 兼容初始 Schema
-- 目标：全新安装与 V1 老库走同一条迁移路径。
-- 先建立 V1 表结构（幂等），后续 migration 逐步演进到 V2。

CREATE TABLE IF NOT EXISTS requests (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    method TEXT NOT NULL,
    url TEXT NOT NULL,
    params TEXT NOT NULL DEFAULT '[]',
    headers TEXT NOT NULL DEFAULT '[]',
    body TEXT NOT NULL,
    auth TEXT NOT NULL,
    collection_id TEXT,
    status TEXT NOT NULL DEFAULT 'draft',
    last_response TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS history (
    id TEXT PRIMARY KEY,
    request TEXT NOT NULL,
    response TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS favorites (
    id TEXT PRIMARY KEY,
    request TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS collections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    request_ids TEXT NOT NULL DEFAULT '[]',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS environments (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL DEFAULT '',
    vars TEXT NOT NULL DEFAULT '[]',
    active INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
