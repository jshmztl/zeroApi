-- 005_environment_secret: Secret 支持
-- Secret 与普通变量分离（VariableKind: plain | secret）。
-- 变量本体仍存储在 environments.vars JSON 中（带 kind 字段）。
-- env_secret_refs 预留 Secret 引用表：后续接入 Windows Credential Manager / DPAPI 时，
-- SQLite 只保存引用，不保存明文（见 ZEROAPI_V2_IMPLEMENTATION.md §11）。
-- environments 的 project_id 列由 Rust 迁移（add_environment_project_id）添加。

CREATE TABLE IF NOT EXISTS env_secret_refs (
    id TEXT PRIMARY KEY,
    env_id TEXT NOT NULL,
    variable_name TEXT NOT NULL,
    secret_ref TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(env_id) REFERENCES environments(id)
);
