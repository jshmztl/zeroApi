//! 版本化 Migration 执行器与数据迁移
//!
//! 两种迁移类型：
//! - SQL：schema 变更（来自 `migrations/*.sql`，编译期嵌入）
//! - Rust：旧数据 JSON 结构转换（V1 → V2，serde 可靠处理）
//!
//! 每个迁移在事务内执行；成功后在 `schema_migrations` 表记录版本。
//! 迁移列表按定义顺序执行，不修改已应用的历史迁移。

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::AppResult;

/// 版本化迁移
enum Migration {
    Sql {
        version: i64,
        name: &'static str,
        sql: &'static str,
    },
    Rust {
        version: i64,
        name: &'static str,
        func: fn(&Connection) -> AppResult<()>,
    },
}

impl Migration {
    fn version(&self) -> i64 {
        match self {
            Migration::Sql { version, .. } | Migration::Rust { version, .. } => *version,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Migration::Sql { name, .. } | Migration::Rust { name, .. } => name,
        }
    }

    fn apply(&self, conn: &Connection) -> AppResult<()> {
        match self {
            Migration::Sql { sql, .. } => conn.execute_batch(sql)?,
            Migration::Rust { func, .. } => func(conn)?,
        }
        Ok(())
    }
}

/// 迁移列表（顺序即执行顺序）
const MIGRATIONS: &[Migration] = &[
    // ---- Schema（SQL）----
    Migration::Sql {
        version: 1,
        name: "001_initial",
        sql: include_str!("../../migrations/001_initial.sql"),
    },
    Migration::Sql {
        version: 2,
        name: "002_projects",
        sql: include_str!("../../migrations/002_projects.sql"),
    },
    // ---- 数据：V1 collections → V2（加 project_id/sort_order，去 request_ids）----
    Migration::Rust {
        version: 3,
        name: "migrate_collections_v2",
        func: migrate_collections_v2,
    },
    Migration::Sql {
        version: 4,
        name: "003_folders",
        sql: include_str!("../../migrations/003_folders.sql"),
    },
    // ---- 数据：V1 requests → V2（headers/params JSON 结构转换，移除 status/last_response）----
    Migration::Rust {
        version: 5,
        name: "migrate_requests_v2",
        func: migrate_requests_v2,
    },
    Migration::Sql {
        version: 6,
        name: "004_request_execution",
        sql: include_str!("../../migrations/004_request_execution.sql"),
    },
    // ---- 数据：history → requests + request_executions（历史不丢）----
    Migration::Rust {
        version: 7,
        name: "migrate_history_to_executions",
        func: migrate_history_to_executions,
    },
    // ---- 数据：favorites.request JSON 转 V2 格式 ----
    Migration::Rust {
        version: 8,
        name: "migrate_favorites_format",
        func: migrate_favorites_format,
    },
    Migration::Sql {
        version: 9,
        name: "005_environment_secret",
        sql: include_str!("../../migrations/005_environment_secret.sql"),
    },
    // ---- 数据：environments 加 project_id 列 ----
    Migration::Rust {
        version: 10,
        name: "add_environment_project_id",
        func: add_environment_project_id,
    },
    Migration::Sql {
        version: 11,
        name: "006_indexes",
        sql: include_str!("../../migrations/006_indexes.sql"),
    },
];

/// 执行全部未应用的迁移
pub(crate) fn run(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at INTEGER NOT NULL
        );",
    )?;

    let applied: std::collections::HashSet<i64> = conn
        .prepare("SELECT version FROM schema_migrations")?
        .query_map([], |r| r.get::<_, i64>(0))?
        .collect::<rusqlite::Result<_>>()?;

    for m in MIGRATIONS {
        if applied.contains(&m.version()) {
            continue;
        }
        log::info!("应用迁移 {} ({})", m.version(), m.name());
        let tx = conn.unchecked_transaction()?;
        m.apply(&tx)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
            params![m.version(), m.name(), Utc::now().timestamp_millis()],
        )?;
        tx.commit()?;
    }
    Ok(())
}

// ========================================================================
// 数据迁移（V1 → V2）
// ========================================================================

/// 未分类集合：未归属任何集合的请求（历史迁移、临时请求）统一挂载
pub(crate) const UNCATEGORIZED_COLLECTION: &str = "uncategorized";

fn table_exists(conn: &Connection, name: &str) -> AppResult<bool> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
        params![name],
        |r| r.get(0),
    )?;
    Ok(n > 0)
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> AppResult<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let names = stmt.query_map([], |r| r.get::<_, String>(1))?;
    for name in names {
        if name? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

/// 确保未分类集合存在（幂等）
fn ensure_uncategorized_collection(conn: &Connection) -> AppResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO collections (id, project_id, name, description, sort_order, created_at, updated_at)
         VALUES (?1, 'default', '未分类', NULL, 0, 0, 0)",
        params![UNCATEGORIZED_COLLECTION],
    )?;
    Ok(())
}

/// V1 collections（含 request_ids JSON）→ V2 collections_v2，并切换表名
fn migrate_collections_v2(conn: &Connection) -> AppResult<()> {
    if !table_exists(conn, "collections_v2")? {
        return Ok(());
    }
    if table_exists(conn, "collections")? {
        conn.execute(
            "INSERT OR IGNORE INTO collections_v2 (id, project_id, name, description, sort_order, created_at, updated_at)
             SELECT id, 'default', name, NULLIF(description, ''), 0, created_at, updated_at FROM collections",
            [],
        )?;
        conn.execute("DROP TABLE IF EXISTS collections", [])?;
    }
    conn.execute("ALTER TABLE collections_v2 RENAME TO collections", [])?;
    ensure_uncategorized_collection(conn)?;
    Ok(())
}

/// V1 requests（headers/params 为 {key,value,enabled} JSON）→ V2 requests_v2，并切换表名
fn migrate_requests_v2(conn: &Connection) -> AppResult<()> {
    if !table_exists(conn, "requests_v2")? {
        return Ok(());
    }
    if table_exists(conn, "requests")? {
        let mut stmt = conn.prepare(
            "SELECT id, collection_id, name, method, url, params, headers, body, auth, created_at, updated_at FROM requests",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, i64>(9)?,
                row.get::<_, i64>(10)?,
            ))
        })?;
        let mut items: Vec<(String, Option<String>, String, String, String, String, String, String, String, i64, i64)> = Vec::new();
        for r in rows {
            items.push(r?);
        }
        drop(stmt);

        let mut ins = conn.prepare(
            "INSERT OR IGNORE INTO requests_v2 (id, collection_id, folder_id, name, method, url, headers, query_params, body, auth, sort_order, created_at, updated_at)
             VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, ?11)",
        )?;
        for (id, cid, name, method, url, params, headers, body, auth, created, updated) in items {
            let cid = cid
                .filter(|c| !c.is_empty())
                .unwrap_or_else(|| UNCATEGORIZED_COLLECTION.to_string());
            let v1 = serde_json::json!({
                "headers": serde_json::from_str::<serde_json::Value>(&headers).unwrap_or_else(|_| serde_json::json!([])),
                "params": serde_json::from_str::<serde_json::Value>(&params).unwrap_or_else(|_| serde_json::json!([])),
                "body": serde_json::from_str::<serde_json::Value>(&body).unwrap_or_else(|_| serde_json::json!({"type":"none"})),
                "auth": serde_json::from_str::<serde_json::Value>(&auth).unwrap_or_else(|_| serde_json::json!({"type":"none"})),
            });
            let (new_headers, new_query, new_body, new_auth) = convert_v1_request_json(&v1);
            ins.execute(params![
                id, cid, name, method, url, new_headers, new_query, new_body, new_auth, created, updated
            ])?;
        }
        drop(ins);
        conn.execute("DROP TABLE IF EXISTS requests", [])?;
    }
    conn.execute("ALTER TABLE requests_v2 RENAME TO requests", [])?;
    Ok(())
}

/// V1 history → requests（hist-* 行）+ request_executions（exec-* 行）
fn migrate_history_to_executions(conn: &Connection) -> AppResult<()> {
    if !table_exists(conn, "history")? || !table_exists(conn, "request_executions")? {
        return Ok(());
    }
    // 幂等：已迁移过则跳过
    let existing: i64 = conn.query_row("SELECT COUNT(*) FROM request_executions", [], |r| r.get(0))?;
    if existing > 0 {
        return Ok(());
    }

    ensure_uncategorized_collection(conn)?;

    let mut stmt = conn.prepare("SELECT id, request, response, created_at FROM history")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
        ))
    })?;
    let mut items: Vec<(String, String, String, i64)> = Vec::new();
    for r in rows {
        items.push(r?);
    }
    drop(stmt);

    let mut ins_req = conn.prepare(
        "INSERT OR IGNORE INTO requests (id, collection_id, folder_id, name, method, url, headers, query_params, body, auth, sort_order, created_at, updated_at)
         VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, ?10)",
    )?;
    let mut ins_exec = conn.prepare(
        "INSERT OR IGNORE INTO request_executions (id, request_id, method, url, status_code, duration_ms, size_bytes, content_type, success, error_code, error_message, response, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, NULL, ?10, ?11)",
    )?;

    for (hid, req_json, resp_json, created) in items {
        let rid = format!("hist-{}", hid);
        let eid = format!("exec-{}", hid);

        let v1req: serde_json::Value =
            serde_json::from_str(&req_json).unwrap_or_else(|_| serde_json::json!({}));
        let name = v1req["name"].as_str().unwrap_or("").to_string();
        let method = v1req["method"].as_str().unwrap_or("GET").to_string();
        let url = v1req["url"].as_str().unwrap_or("").to_string();
        let (new_headers, new_query, new_body, new_auth) = convert_v1_request_json(&v1req);
        ins_req.execute(params![rid, UNCATEGORIZED_COLLECTION, name, method, url, new_headers, new_query, new_body, new_auth, created])?;

        // V1 ResponseSnapshot → V2
        let v1resp: serde_json::Value =
            serde_json::from_str(&resp_json).unwrap_or_else(|_| serde_json::json!({}));
        let status = v1resp["status"].as_u64().unwrap_or(0);
        let duration = v1resp["time_ms"].as_u64().unwrap_or(0);
        let size = v1resp["size_bytes"].as_u64().unwrap_or(0);
        let success = (200..400).contains(&status);
        let v2resp = serde_json::json!({
            "status": status,
            "status_text": v1resp["status_text"].as_str().unwrap_or(""),
            "headers": v1resp["headers"].as_object().map(|m| {
                m.iter()
                    .map(|(k, v)| serde_json::json!({"name": k, "value": v.as_str().unwrap_or("")}))
                    .collect::<Vec<_>>()
            }).unwrap_or_default(),
            "body": {
                "type": "text",
                "text": v1resp["body"].as_str().unwrap_or(""),
            },
            "size_bytes": size,
            "content_type": v1resp["content_type"].as_str().filter(|s| !s.is_empty()),
            "timing": {
                "dns_ms": serde_json::Value::Null,
                "tcp_ms": serde_json::Value::Null,
                "tls_ms": serde_json::Value::Null,
                "request_ms": serde_json::Value::Null,
                "response_ms": serde_json::Value::Null,
                "total_ms": duration,
            },
        });
        let content_type = v1resp["content_type"].as_str().map(|s| s.to_string());
        ins_exec.execute(params![
            eid, rid, method, url,
            status as u16, duration as u64, size as u64, content_type,
            success as i32, serde_json::to_string(&v2resp)?, created,
        ])?;
    }
    Ok(())
}

/// V1 favorites.request JSON → V2 格式（headers/params 结构转换）
fn migrate_favorites_format(conn: &Connection) -> AppResult<()> {
    if !table_exists(conn, "favorites")? {
        return Ok(());
    }
    let mut stmt = conn.prepare("SELECT id, request FROM favorites")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut items: Vec<(String, String)> = Vec::new();
    for r in rows {
        items.push(r?);
    }
    drop(stmt);

    let mut upd = conn.prepare("UPDATE favorites SET request = ?1 WHERE id = ?2")?;
    for (id, req_json) in items {
        let v1: serde_json::Value =
            serde_json::from_str(&req_json).unwrap_or_else(|_| serde_json::json!({}));
        let v2 = build_v2_request_json(&v1);
        upd.execute(params![serde_json::to_string(&v2)?, id])?;
    }
    Ok(())
}

/// environments 加 project_id 列
fn add_environment_project_id(conn: &Connection) -> AppResult<()> {
    if table_exists(conn, "environments")? && !column_exists(conn, "environments", "project_id")? {
        conn.execute("ALTER TABLE environments ADD COLUMN project_id TEXT", [])?;
    }
    Ok(())
}

// ========================================================================
// V1 → V2 JSON 结构转换
// ========================================================================

/// 把 V1 Request JSON 的 headers/params/body/auth 转换为 V2 结构
/// 返回 (headers, query, body, auth) 的 JSON 字符串；body/auth 为 None 表示无。
fn convert_v1_request_json(
    v1: &serde_json::Value,
) -> (String, String, Option<String>, Option<String>) {
    let headers: Vec<serde_json::Value> = v1
        .get("headers")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|x| {
                    serde_json::json!({
                        "name": x["key"].as_str().unwrap_or(""),
                        "value": x["value"].as_str().unwrap_or(""),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let query: Vec<serde_json::Value> = v1
        .get("params")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|x| {
                    serde_json::json!({
                        "name": x["key"].as_str().unwrap_or(""),
                        "value": x["value"].as_str().unwrap_or(""),
                        "enabled": x["enabled"].as_bool().unwrap_or(true),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let body = convert_optional_enum(v1.get("body"), "none");
    let auth = convert_optional_enum(v1.get("auth"), "none");

    (
        serde_json::to_string(&headers).unwrap_or_else(|_| "[]".into()),
        serde_json::to_string(&query).unwrap_or_else(|_| "[]".into()),
        body,
        auth,
    )
}

/// type == none_tag 的枚举体视为 None
fn convert_optional_enum(v: Option<&serde_json::Value>, none_tag: &str) -> Option<String> {
    let v = v?;
    if v.get("type").and_then(|t| t.as_str()) == Some(none_tag) {
        None
    } else {
        Some(serde_json::to_string(v).unwrap_or_else(|_| "{}".into()))
    }
}

/// 构造完整的 V2 Request JSON
fn build_v2_request_json(v1: &serde_json::Value) -> serde_json::Value {
    let (headers, query, body, auth) = convert_v1_request_json(v1);
    let cid = v1["collection_id"]
        .as_str()
        .filter(|c| !c.is_empty())
        .unwrap_or(UNCATEGORIZED_COLLECTION);
    serde_json::json!({
        "id": v1["id"].as_str().unwrap_or(""),
        "collection_id": cid,
        "folder_id": serde_json::Value::Null,
        "name": v1["name"].as_str().unwrap_or(""),
        "method": v1["method"].as_str().unwrap_or("GET"),
        "url": v1["url"].as_str().unwrap_or(""),
        "headers": serde_json::from_str::<serde_json::Value>(&headers).unwrap_or_else(|_| serde_json::json!([])),
        "query": serde_json::from_str::<serde_json::Value>(&query).unwrap_or_else(|_| serde_json::json!([])),
        "body": body.and_then(|b| serde_json::from_str::<serde_json::Value>(&b).ok()).unwrap_or(serde_json::Value::Null),
        "auth": auth.and_then(|a| serde_json::from_str::<serde_json::Value>(&a).ok()).unwrap_or(serde_json::Value::Null),
        "sort_order": 0,
        "created_at": v1["created_at"].as_i64().unwrap_or(0),
        "updated_at": v1["updated_at"].as_i64().unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 模拟真实 V1 数据库（表结构与 V1 代码一致）
    fn setup_v1_db(conn: &Connection) {
        conn.execute_batch(
            "CREATE TABLE requests (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, method TEXT NOT NULL, url TEXT NOT NULL,
                params TEXT NOT NULL DEFAULT '[]', headers TEXT NOT NULL DEFAULT '[]',
                body TEXT NOT NULL, auth TEXT NOT NULL, collection_id TEXT,
                created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
             );
             CREATE TABLE history (
                id TEXT PRIMARY KEY, request TEXT NOT NULL, response TEXT NOT NULL, created_at INTEGER NOT NULL
             );
             CREATE TABLE favorites (
                id TEXT PRIMARY KEY, request TEXT NOT NULL, created_at INTEGER NOT NULL
             );
             CREATE TABLE collections (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT NOT NULL DEFAULT '',
                request_ids TEXT NOT NULL DEFAULT '[]', created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
             );
             CREATE TABLE environments (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, base_url TEXT NOT NULL DEFAULT '',
                vars TEXT NOT NULL DEFAULT '[]', active INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )
        .unwrap();
        // 集合
        conn.execute(
            "INSERT INTO collections (id, name, description, request_ids, created_at, updated_at) VALUES ('col1', '用户', '', '[\"req1\"]', 1, 1)",
            [],
        )
        .unwrap();
        // 保存的请求（V1 headers/params 为 {key,value,enabled}）
        conn.execute(
            "INSERT INTO requests (id, name, method, url, params, headers, body, auth, collection_id, created_at, updated_at)
             VALUES ('req1', '获取用户', 'GET', 'https://api.example.com/users',
                     '[{\"key\":\"page\",\"value\":\"1\",\"enabled\":true}]',
                     '[{\"key\":\"Accept\",\"value\":\"application/json\",\"enabled\":true}]',
                     '{\"type\":\"none\"}', '{\"type\":\"none\"}', 'col1', 1, 1)",
            [],
        )
        .unwrap();
        // 历史（V1 HistoryItem: request/response JSON）
        conn.execute(
            "INSERT INTO history (id, request, response, created_at) VALUES ('h1',
              '{\"id\":\"\",\"name\":\"历史请求\",\"method\":\"POST\",\"url\":\"https://api.example.com\",\"params\":[],\"headers\":[{\"key\":\"Content-Type\",\"value\":\"application/json\",\"enabled\":true}],\"body\":{\"type\":\"raw\",\"content_type\":\"application/json\",\"content\":\"{}\"},\"auth\":{\"type\":\"none\"},\"collection_id\":null}',
              '{\"status\":200,\"status_text\":\"OK\",\"headers\":{\"content-type\":\"application/json\"},\"body\":\"{}\",\"time_ms\":42,\"size_bytes\":2,\"content_type\":\"application/json\"}',
              100)",
            [],
        )
        .unwrap();
        // 收藏
        conn.execute(
            "INSERT INTO favorites (id, request, created_at) VALUES ('f1',
              '{\"id\":\"fav1\",\"name\":\"收藏\",\"method\":\"GET\",\"url\":\"https://x.com\",\"params\":[],\"headers\":[{\"key\":\"A\",\"value\":\"b\",\"enabled\":true}],\"body\":{\"type\":\"none\"},\"auth\":{\"type\":\"none\"},\"collection_id\":null}',
              200)",
            [],
        )
        .unwrap();
    }

    #[test]
    fn test_migrate_v1_to_v2() {
        let conn = Connection::open_in_memory().unwrap();
        setup_v1_db(&conn);
        run(&conn).unwrap();

        // 集合：col1 + uncategorized，col1 归属默认项目
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM collections", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 2);
        let pid: String = conn
            .query_row("SELECT project_id FROM collections WHERE id='col1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(pid, "default");

        // requests 表已切换为 V2 结构，数据保留
        let cnt: i64 = conn
            .query_row("SELECT COUNT(*) FROM requests WHERE id='req1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(cnt, 1);

        // 历史 → request_executions（exec-h1）与 requests（hist-h1）
        let exec: i64 = conn
            .query_row("SELECT COUNT(*) FROM request_executions WHERE id='exec-h1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(exec, 1);
        let hist_req: i64 = conn
            .query_row("SELECT COUNT(*) FROM requests WHERE id='hist-h1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(hist_req, 1);
        // 执行记录内容（status_code / duration_ms / response JSON）
        let (code, dur): (Option<u16>, i64) = conn
            .query_row(
                "SELECT status_code, duration_ms FROM request_executions WHERE id='exec-h1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(code, Some(200));
        assert_eq!(dur, 42);

        // 收藏 request JSON 已转 V2（query 字段存在、params 不再出现）
        let fav_json: String = conn
            .query_row("SELECT request FROM favorites WHERE id='f1'", [], |r| r.get(0))
            .unwrap();
        assert!(fav_json.contains("\"query\""));
        assert!(!fav_json.contains("\"params\""));
        assert!(fav_json.contains("\"collection_id\":\"uncategorized\""));

        // 环境表有 project_id 列
        assert!(column_exists(&conn, "environments", "project_id").unwrap());

        // 幂等：重复执行不报错、不重复数据
        run(&conn).unwrap();
        let exec2: i64 = conn
            .query_row("SELECT COUNT(*) FROM request_executions WHERE id='exec-h1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(exec2, 1);
    }

    #[test]
    fn test_fresh_db_migrates_cleanly() {
        // 全新安装：空库直接跑迁移
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();
        let tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('projects','collections','folders','requests','request_executions','environments','favorites')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(tables, 7);
    }
}
