//! Request 持久化（含收藏）

use std::sync::Arc;

use rusqlite::params;

use crate::db::Database;
use crate::domain::{Favorite, HttpMethod, Request};
use crate::AppResult;

#[derive(Clone)]
pub struct RequestRepo {
    db: Arc<Database>,
}

impl RequestRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 列出请求：可按集合 / 文件夹过滤
    pub fn list(
        &self,
        collection_id: Option<&str>,
        folder_id: Option<&str>,
    ) -> AppResult<Vec<Request>> {
        let conn = self.db.conn();
        let mut out = Vec::new();
        match (collection_id, folder_id) {
            (Some(cid), Some(fid)) => {
                let mut stmt = conn.prepare(
                    "SELECT id, collection_id, folder_id, name, method, url, headers, query_params, body, auth, sort_order, created_at, updated_at
                     FROM requests WHERE collection_id = ?1 AND folder_id = ?2 ORDER BY sort_order ASC, updated_at DESC",
                )?;
                let rows = stmt.query_map(params![cid, fid], Self::map_row)?;
                for r in rows {
                    out.push(r?);
                }
            }
            (Some(cid), None) => {
                let mut stmt = conn.prepare(
                    "SELECT id, collection_id, folder_id, name, method, url, headers, query_params, body, auth, sort_order, created_at, updated_at
                     FROM requests WHERE collection_id = ?1 ORDER BY sort_order ASC, updated_at DESC",
                )?;
                let rows = stmt.query_map(params![cid], Self::map_row)?;
                for r in rows {
                    out.push(r?);
                }
            }
            _ => {
                let mut stmt = conn.prepare(
                    "SELECT id, collection_id, folder_id, name, method, url, headers, query_params, body, auth, sort_order, created_at, updated_at
                     FROM requests ORDER BY sort_order ASC, updated_at DESC",
                )?;
                let rows = stmt.query_map([], Self::map_row)?;
                for r in rows {
                    out.push(r?);
                }
            }
        }
        Ok(out)
    }

    pub fn get(&self, id: &str) -> AppResult<Option<Request>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, collection_id, folder_id, name, method, url, headers, query_params, body, auth, sort_order, created_at, updated_at
             FROM requests WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::map_row)?;
        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    pub fn upsert(&self, req: &Request) -> AppResult<()> {
        let conn = self.db.conn();
        let headers = serde_json::to_string(&req.headers)?;
        let query = serde_json::to_string(&req.query)?;
        let body = req.body.as_ref().map(|b| serde_json::to_string(b)).transpose()?;
        let auth = req.auth.as_ref().map(|a| serde_json::to_string(a)).transpose()?;
        conn.execute(
            "INSERT INTO requests (id, collection_id, folder_id, name, method, url, headers, query_params, body, auth, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(id) DO UPDATE SET
               collection_id=excluded.collection_id, folder_id=excluded.folder_id,
               name=excluded.name, method=excluded.method, url=excluded.url,
               headers=excluded.headers, query_params=excluded.query_params,
               body=excluded.body, auth=excluded.auth, sort_order=excluded.sort_order,
               updated_at=excluded.updated_at",
            params![
                req.id, req.collection_id, req.folder_id, req.name,
                req.method.as_str(), req.url, headers, query, body, auth,
                req.sort_order, req.created_at, req.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        let conn = self.db.conn();
        conn.execute("DELETE FROM request_executions WHERE request_id = ?1", params![id])?;
        conn.execute("DELETE FROM requests WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ---------- 收藏 ----------

    pub fn add_favorite(&self, fav: &Favorite) -> AppResult<()> {
        let conn = self.db.conn();
        let req = serde_json::to_string(&fav.request)?;
        conn.execute(
            "INSERT OR REPLACE INTO favorites (id, request, created_at) VALUES (?1, ?2, ?3)",
            params![fav.id, req, fav.created_at],
        )?;
        Ok(())
    }

    pub fn remove_favorite(&self, id: &str) -> AppResult<()> {
        let conn = self.db.conn();
        conn.execute("DELETE FROM favorites WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_favorites(&self) -> AppResult<Vec<Favorite>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, request, created_at FROM favorites ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (id, req_json, created_at) = r?;
            let request: Request = serde_json::from_str(&req_json)?;
            out.push(Favorite {
                id,
                request,
                created_at,
            });
        }
        Ok(out)
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Request> {
        let headers_str: String = row.get(6)?;
        let query_str: String = row.get(7)?;
        let body_str: Option<String> = row.get(8)?;
        let auth_str: Option<String> = row.get(9)?;
        let method_str: String = row.get(4)?;
        let method = HttpMethod::from_str(&method_str).unwrap_or(HttpMethod::Get);
        Ok(Request {
            id: row.get(0)?,
            collection_id: row.get(1)?,
            folder_id: row.get(2)?,
            name: row.get(3)?,
            method,
            url: row.get(5)?,
            headers: serde_json::from_str(&headers_str).unwrap_or_default(),
            query: serde_json::from_str(&query_str).unwrap_or_default(),
            body: body_str
                .and_then(|s| serde_json::from_str(&s).ok()),
            auth: auth_str.and_then(|s| serde_json::from_str(&s).ok()),
            sort_order: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    }
}
