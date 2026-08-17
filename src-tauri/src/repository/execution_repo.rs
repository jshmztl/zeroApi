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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::domain::{HeaderEntry, HttpMethod, ResponseBody, Timing};
    use std::sync::Arc;

    fn setup() -> ExecutionRepo {
        let dir = std::env::temp_dir().join(format!("zeroapi-exec-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = Arc::new(Database::new(dir.join("test.db")).unwrap());
        db.migrate().unwrap();
        // 插入请求行，满足 request_executions 的外键约束
        {
            let conn = db.conn();
            conn.execute(
                "INSERT INTO requests (id, collection_id, name, method, url, headers, query_params, body, auth, sort_order, created_at, updated_at)
                 VALUES ('r1', 'uncategorized', '测试请求', 'GET', 'https://api.example.com', '[]', '[]', NULL, NULL, 0, 0, 0)",
                [],
            )
            .unwrap();
        }
        ExecutionRepo::new(db)
    }

    fn sample_response(status: u16) -> ResponseSnapshot {
        ResponseSnapshot {
            status,
            status_text: "OK".into(),
            headers: vec![HeaderEntry::new("content-type", "application/json")],
            body: ResponseBody::Text("{\"ok\":true}".into()),
            size_bytes: 12,
            content_type: Some("application/json".into()),
            timing: Timing::total(42),
        }
    }

    #[test]
    fn test_insert_and_list() {
        let repo = setup();
        let ok = RequestExecution::success("e1".into(), "r1".into(), 100, 42, sample_response(200));
        repo.insert(&ok, "GET", "https://api.example.com").unwrap();
        let fail = RequestExecution::failure(
            "e2".into(),
            "r1".into(),
            200,
            3000,
            NetworkError::new(NetworkErrorKind::Timeout, "timeout"),
        );
        repo.insert(&fail, "POST", "https://api.example.com").unwrap();

        let all = repo.list(100, None).unwrap();
        assert_eq!(all.len(), 2);
        // 按时间倒序
        assert_eq!(all[0].id, "e2");
        assert_eq!(all[0].success, false);
        assert_eq!(all[0].error.as_ref().unwrap().kind, NetworkErrorKind::Timeout);

        // 按请求过滤
        let by_req = repo.list(100, Some("r1")).unwrap();
        assert_eq!(by_req.len(), 2);
        assert!(by_req.iter().any(|e| e.status_code == Some(200)));
        assert!(by_req.iter().any(|e| e.response.is_some()));

        // 摘要列表带名称（对应 requests 行存在 → 名称正确）
        let summary = repo.list_summary(100).unwrap();
        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0].method, HttpMethod::Post);
        assert_eq!(summary[0].url, "https://api.example.com");
        assert_eq!(summary[0].name, "测试请求");
    }

    #[test]
    fn test_prune_history_limit() {
        let repo = setup();
        for i in 0..10 {
            let e = RequestExecution::success(
                format!("e{}", i),
                "r1".into(),
                i * 100,
                1,
                sample_response(200),
            );
            repo.insert(&e, "GET", "https://api.example.com").unwrap();
        }
        // 只保留最近 3 条（created_at 最大）
        repo.prune(3).unwrap();
        let all = repo.list(100, None).unwrap();
        assert_eq!(all.len(), 3);
        // e7 e8 e9（created_at 700/800/900）
        assert!(all.iter().any(|e| e.id == "e9"));
        assert!(!all.iter().any(|e| e.id == "e0"));
    }
}
