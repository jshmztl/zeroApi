//! RequestExecution 持久化（V2 历史）

use std::sync::Arc;

use rusqlite::params;

use crate::db::Database;
use crate::domain::{ExecutionListItem, NetworkError, NetworkErrorKind, RequestExecution, ResponseSnapshot};
use crate::AppResult;

#[derive(Clone)]
pub struct ExecutionRepo {
    db: Arc<Database>,
}

impl ExecutionRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 插入执行记录；method/url 为摘要列（冗余保存请求的方法与地址，供列表展示）
    pub fn insert(&self, e: &RequestExecution, method: &str, url: &str) -> AppResult<()> {
        let conn = self.db.conn();
        let response = e
            .response
            .as_ref()
            .map(|r| serde_json::to_string(r))
            .transpose()?;
        let error_code = e.error.as_ref().map(|er| er.kind.as_str().to_string());
        let error_message = e.error.as_ref().map(|er| er.message.clone());
        conn.execute(
            "INSERT OR IGNORE INTO request_executions
             (id, request_id, method, url, status_code, duration_ms, size_bytes, content_type, success, error_code, error_message, response, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                e.id, e.request_id, method, url,
                e.status_code, e.duration_ms as i64,
                e.response.as_ref().map(|r| r.size_bytes as i64),
                e.response.as_ref().and_then(|r| r.content_type.clone()),
                e.success as i32, error_code, error_message, response, e.started_at
            ],
        )?;
        Ok(())
    }

    pub fn list(&self, limit: u32, request_id: Option<&str>) -> AppResult<Vec<RequestExecution>> {
        let conn = self.db.conn();
        let mut out = Vec::new();
        match request_id {
            Some(rid) => {
                let mut stmt = conn.prepare(
                    "SELECT id, request_id, method, url, status_code, duration_ms, size_bytes, content_type, success, error_code, error_message, response, created_at
                     FROM request_executions WHERE request_id = ?1 ORDER BY created_at DESC LIMIT ?2",
                )?;
                let rows = stmt.query_map(params![rid, limit], Self::map_row)?;
                for r in rows {
                    out.push(r?);
                }
            }
            None => {
                let mut stmt = conn.prepare(
                    "SELECT id, request_id, method, url, status_code, duration_ms, size_bytes, content_type, success, error_code, error_message, response, created_at
                     FROM request_executions ORDER BY created_at DESC LIMIT ?1",
                )?;
                let rows = stmt.query_map(params![limit], Self::map_row)?;
                for r in rows {
                    out.push(r?);
                }
            }
        }
        Ok(out)
    }

    /// 历史列表摘要（带请求名称，供前端列表展示）
    pub fn list_summary(&self, limit: u32) -> AppResult<Vec<ExecutionListItem>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT e.id, e.request_id, e.method, e.url, e.status_code, e.duration_ms, e.size_bytes,
                    e.content_type, e.success, e.error_code, e.error_message, e.response, e.created_at,
                    COALESCE(r.name, '')
             FROM request_executions e
             LEFT JOIN requests r ON r.id = e.request_id
             ORDER BY e.created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            let (exec, method, url, name) = Self::map_row_full(row)?;
            Ok((exec, method, url, name))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (exec, method, url, name) = r?;
            out.push(ExecutionListItem {
                execution: exec,
                method: crate::domain::HttpMethod::from_str(&method)
                    .unwrap_or(crate::domain::HttpMethod::Get),
                url,
                name,
            });
        }
        Ok(out)
    }

    pub fn get(&self, id: &str) -> AppResult<Option<RequestExecution>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, request_id, method, url, status_code, duration_ms, size_bytes, content_type, success, error_code, error_message, response, created_at
             FROM request_executions WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::map_row)?;
        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        let conn = self.db.conn();
        conn.execute("DELETE FROM request_executions WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear(&self) -> AppResult<()> {
        let conn = self.db.conn();
        conn.execute("DELETE FROM request_executions", [])?;
        Ok(())
    }

    pub fn count(&self) -> AppResult<i64> {
        let conn = self.db.conn();
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM request_executions", [], |r| r.get(0))?;
        Ok(n)
    }

    /// 裁剪历史：仅保留最近 keep 条
    pub fn prune(&self, keep: u32) -> AppResult<()> {
        let conn = self.db.conn();
        conn.execute(
            "DELETE FROM request_executions WHERE id NOT IN (
                SELECT id FROM request_executions ORDER BY created_at DESC LIMIT ?1
            )",
            params![keep as i64],
        )?;
        Ok(())
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<RequestExecution> {
        let (exec, _, _, _) = Self::map_row_full(row)?;
        Ok(exec)
    }

    /// 完整映射，额外返回 method/url（摘要列）与 name
    fn map_row_full(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(RequestExecution, String, String, String)> {
        let id: String = row.get(0)?;
        let request_id: String = row.get(1)?;
        let method: String = row.get(2)?;
        let url: String = row.get(3)?;
        let status_code: Option<u16> = row.get(4)?;
        let duration_ms: Option<i64> = row.get(5)?;
        let success: i32 = row.get(8)?;
        let error_code: Option<String> = row.get(9)?;
        let error_message: Option<String> = row.get(10)?;
        let response_str: Option<String> = row.get(11)?;
        let created_at: i64 = row.get(12)?;
        let name: String = row.get(13).unwrap_or_default();

        let response: Option<ResponseSnapshot> = response_str
            .and_then(|s| serde_json::from_str(&s).ok());
        let error = match (error_code, error_message) {
            (Some(code), Some(message)) => {
                let kind = match code.as_str() {
                    "invalid_url" => NetworkErrorKind::InvalidUrl,
                    "dns" => NetworkErrorKind::Dns,
                    "connection" => NetworkErrorKind::Connection,
                    "timeout" => NetworkErrorKind::Timeout,
                    "tls" => NetworkErrorKind::Tls,
                    "proxy" => NetworkErrorKind::Proxy,
                    "protocol" => NetworkErrorKind::Protocol,
                    "cancelled" => NetworkErrorKind::Cancelled,
                    _ => NetworkErrorKind::Unknown,
                };
                Some(NetworkError {
                    kind,
                    message,
                    detail: None,
                })
            }
            _ => None,
        };

        Ok((
            RequestExecution {
                id,
                request_id,
                started_at: created_at,
                duration_ms: duration_ms.unwrap_or(0) as u64,
                status_code,
                success: success != 0,
                error,
                response,
            },
            method,
            url,
            name,
        ))
    }
}
