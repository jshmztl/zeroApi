//! SQLite 持久化层
use std::path::Path;
use std::sync::Mutex;

use crate::models::*;
use crate::AppResult;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde_json;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: impl AsRef<Path>) -> AppResult<Self> {
        let conn = Connection::open(path)?;
        // 启用外键 & WAL
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn migrate(&self) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
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
                vars TEXT NOT NULL DEFAULT '[]',
                active INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_history_created_at ON history(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_favorites_created_at ON favorites(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_collections_updated_at ON collections(updated_at DESC);
            "#,
        )?;
        // 迁移：为老版本数据库添加 status 列
        Self::ensure_column(&conn, "requests", "status",
            "ALTER TABLE requests ADD COLUMN status TEXT NOT NULL DEFAULT 'draft'")?;
        // 迁移：为老版本数据库添加 base_url 列
        Self::ensure_column(&conn, "environments", "base_url",
            "ALTER TABLE environments ADD COLUMN base_url TEXT NOT NULL DEFAULT ''")?;
        Ok(())
    }

    /// 检查列是否存在，不存在则执行 ALTER
    fn ensure_column(conn: &Connection, table: &str, column: &str, ddl: &str) -> AppResult<()> {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
        let exists = stmt.query_map([], |r| r.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .any(|c| c == column);
        if !exists {
            log::info!("执行数据库迁移: {}", ddl);
            conn.execute(ddl, [])?;
        }
        Ok(())
    }

    fn now() -> i64 {
        Utc::now().timestamp_millis()
    }

    // ---------- Settings ----------

    pub fn get_settings(&self) -> AppResult<Settings> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'app'")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let raw: String = row.get(0)?;
            Ok(serde_json::from_str(&raw).unwrap_or_default())
        } else {
            Ok(Settings::default())
        }
    }

    pub fn save_settings(&self, settings: &Settings) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        let raw = serde_json::to_string(settings)?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('app', ?1)",
            params![raw],
        )?;
        Ok(())
    }

    // ---------- History ----------

    pub fn add_history(&self, item: &HistoryItem) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        let req = serde_json::to_string(&item.request)?;
        let resp = serde_json::to_string(&item.response)?;
        conn.execute(
            "INSERT INTO history (id, request, response, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![item.id, req, resp, item.created_at],
        )?;
        Ok(())
    }

    pub fn list_history(&self, limit: u32) -> AppResult<Vec<HistoryItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, request, response, created_at FROM history ORDER BY created_at DESC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit], |row| {
            let id: String = row.get(0)?;
            let req: String = row.get(1)?;
            let resp: String = row.get(2)?;
            let created_at: i64 = row.get(3)?;
            Ok((id, req, resp, created_at))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (id, req, resp, created_at) = r?;
            let request: Request = serde_json::from_str(&req)?;
            let response: ResponseSnapshot = serde_json::from_str(&resp)?;
            out.push(HistoryItem { id, request, response, created_at });
        }
        Ok(out)
    }

    pub fn clear_history(&self) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM history", [])?;
        Ok(())
    }

    pub fn delete_history(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM history WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ---------- Favorites ----------

    pub fn add_favorite(&self, fav: &Favorite) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        let req = serde_json::to_string(&fav.request)?;
        conn.execute(
            "INSERT OR REPLACE INTO favorites (id, request, created_at) VALUES (?1, ?2, ?3)",
            params![fav.id, req, fav.created_at],
        )?;
        Ok(())
    }

    pub fn remove_favorite(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM favorites WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_favorites(&self) -> AppResult<Vec<Favorite>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id, request, created_at FROM favorites ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let req: String = row.get(1)?;
            let created_at: i64 = row.get(2)?;
            Ok((id, req, created_at))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (id, req, created_at) = r?;
            let request: Request = serde_json::from_str(&req)?;
            out.push(Favorite { id, request, created_at });
        }
        Ok(out)
    }

    // ---------- Collections ----------

    pub fn upsert_collection(&self, col: &Collection) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        let ids = serde_json::to_string(&col.request_ids)?;
        conn.execute(
            "INSERT OR REPLACE INTO collections (id, name, description, request_ids, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![col.id, col.name, col.description, ids, col.created_at, col.updated_at],
        )?;
        Ok(())
    }

    pub fn delete_collection(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM collections WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_collection(&self, id: &str, patch: &CollectionPatch) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        if let Some(ref name) = patch.name {
            conn.execute("UPDATE collections SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![name, Self::now(), id])?;
        }
        if let Some(ref desc) = patch.description {
            conn.execute("UPDATE collections SET description = ?1, updated_at = ?2 WHERE id = ?3",
                params![desc, Self::now(), id])?;
        }
        if let Some(ref request_ids) = patch.request_ids {
            let ids = serde_json::to_string(request_ids)?;
            conn.execute("UPDATE collections SET request_ids = ?1, updated_at = ?2 WHERE id = ?3",
                params![ids, Self::now(), id])?;
        }
        // Any other collection field update marks updated_at
        conn.execute("UPDATE collections SET updated_at = ?1 WHERE id = ?2",
            params![Self::now(), id])?;
        Ok(())
    }

    pub fn list_collections(&self) -> AppResult<Vec<Collection>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, request_ids, created_at, updated_at FROM collections ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let desc: String = row.get(2)?;
            let ids: String = row.get(3)?;
            let c_at: i64 = row.get(4)?;
            let u_at: i64 = row.get(5)?;
            Ok((id, name, desc, ids, c_at, u_at))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (id, name, desc, ids, c_at, u_at) = r?;
            let request_ids: Vec<String> = serde_json::from_str(&ids).unwrap_or_default();
            out.push(Collection {
                id, name, description: desc, request_ids, created_at: c_at, updated_at: u_at,
            });
        }
        Ok(out)
    }

    // ---------- Saved Requests ----------

    pub fn upsert_saved_request(&self, req: &Request, collection_id: Option<&str>) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        let params = serde_json::to_string(&req.params)?;
        let headers = serde_json::to_string(&req.headers)?;
        let body = serde_json::to_string(&req.body)?;
        let auth = serde_json::to_string(&req.auth)?;
        let now = Self::now();
        conn.execute(
            "INSERT INTO requests (id, name, method, url, params, headers, body, auth, status, collection_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)
             ON CONFLICT(id) DO UPDATE SET
               name=excluded.name, method=excluded.method, url=excluded.url,
               params=excluded.params, headers=excluded.headers, body=excluded.body, auth=excluded.auth,
               status=excluded.status, collection_id=excluded.collection_id, updated_at=excluded.updated_at",
            params![req.id, req.name, req.method, req.url, params, headers, body, auth,
                    req.status.as_str(), collection_id, now],
        )?;
        Ok(())
    }

    pub fn list_saved_requests(&self, collection_id: Option<&str>) -> AppResult<Vec<Request>> {
        let conn = self.conn.lock().unwrap();
        let mut out = Vec::new();

        let sql = "SELECT id, name, method, url, params, headers, body, auth, status, collection_id FROM requests WHERE collection_id = ?1 ORDER BY updated_at DESC";
        let all_sql = "SELECT id, name, method, url, params, headers, body, auth, status, collection_id FROM requests ORDER BY updated_at DESC";

        if let Some(cid) = collection_id {
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(params![cid], |row| Self::map_saved_request(row))?;
            for r in rows {
                out.push(r?);
            }
        } else {
            let mut stmt = conn.prepare(all_sql)?;
            let rows = stmt.query_map([], |row| Self::map_saved_request(row))?;
            for r in rows {
                out.push(r?);
            }
        }
        Ok(out)
    }

    fn map_saved_request(row: &rusqlite::Row) -> rusqlite::Result<Request> {
        let params_str: String = row.get(4)?;
        let headers_str: String = row.get(5)?;
        let body_str: String = row.get(6)?;
        let auth_str: String = row.get(7)?;
        let status_str: String = row.get(8)?;
        let collection_id: Option<String> = row.get(9)?;
        let status: RequestStatus = serde_json::from_str(&format!("\"{}\"", status_str)).unwrap_or_default();
        Ok(Request {
            id: row.get(0)?,
            name: row.get(1)?,
            method: row.get(2)?,
            url: row.get(3)?,
            params: serde_json::from_str(&params_str).unwrap_or_default(),
            headers: serde_json::from_str(&headers_str).unwrap_or_default(),
            body: serde_json::from_str(&body_str).unwrap_or_default(),
            auth: serde_json::from_str(&auth_str).unwrap_or_default(),
            collection_id,
            status,
        })
    }

    // ---------- Environments ----------

    pub fn upsert_environment(&self, env: &Environment) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        let vars = serde_json::to_string(&env.vars)?;
        // 单选 active
        if env.active {
            conn.execute("UPDATE environments SET active = 0", [])?;
        }
        conn.execute(
            "INSERT OR REPLACE INTO environments (id, name, base_url, vars, active) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![env.id, env.name, env.base_url, vars, env.active as i32],
        )?;
        Ok(())
    }

    pub fn delete_environment(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM environments WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_environments(&self) -> AppResult<Vec<Environment>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, base_url, vars, active FROM environments ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let base_url: String = row.get(2)?;
            let vars: String = row.get(3)?;
            let active: i32 = row.get(4)?;
            Ok((id, name, base_url, vars, active != 0))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (id, name, base_url, vars, active) = r?;
            let v: Vec<KeyValue> = serde_json::from_str(&vars).unwrap_or_default();
            out.push(Environment { id, name, base_url, vars: v, active });
        }
        Ok(out)
    }

    // ---------- Maintenance ----------

    pub fn clear_all(&self) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "DELETE FROM history; DELETE FROM favorites; DELETE FROM collections; DELETE FROM environments; DELETE FROM requests; DELETE FROM settings;",
        )?;
        Ok(())
    }
}
