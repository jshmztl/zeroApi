-- 002_projects: Project 领域模型
-- 新增 projects 表，并预置 V2 collections 表结构（含 project_id / sort_order）。
-- 旧 collections 表的数据迁移由 Rust 数据迁移（migrate_collections_v2）完成。

CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    root_path TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 默认项目：V1 数据迁移后的归属项目
INSERT OR IGNORE INTO projects (id, name, description, root_path, created_at, updated_at)
VALUES ('default', '默认项目', '由 V1 数据迁移自动创建', NULL, 0, 0);

-- V2 collections 表结构（数据拷贝与切换由 Rust 迁移完成）
CREATE TABLE IF NOT EXISTS collections_v2 (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL DEFAULT 'default',
    name TEXT NOT NULL,
    description TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY(project_id) REFERENCES projects(id)
);
