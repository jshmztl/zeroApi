//! Collection 持久化

use std::sync::Arc;

use rusqlite::params;

use crate::db::Database;
use crate::domain::Collection;
use crate::AppResult;

#[derive(Clone)]
pub struct CollectionRepo {
    db: Arc<Database>,
}

impl CollectionRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn list(&self, project_id: Option<&str>) -> AppResult<Vec<Collection>> {
        let conn = self.db.conn();
        let mut out = Vec::new();
        match project_id {
            Some(pid) => {
                let mut stmt = conn.prepare(
                    "SELECT id, project_id, name, description, sort_order, created_at, updated_at
                     FROM collections WHERE project_id = ?1 ORDER BY sort_order ASC, updated_at DESC",
                )?;
                let rows = stmt.query_map(params![pid], Self::map_row)?;
                for r in rows {
                    out.push(r?);
                }
            }
            None => {
                let mut stmt = conn.prepare(
                    "SELECT id, project_id, name, description, sort_order, created_at, updated_at
                     FROM collections ORDER BY sort_order ASC, updated_at DESC",
                )?;
                let rows = stmt.query_map([], Self::map_row)?;
                for r in rows {
                    out.push(r?);
                }
            }
        }
        Ok(out)
    }

    pub fn get(&self, id: &str) -> AppResult<Option<Collection>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, description, sort_order, created_at, updated_at
             FROM collections WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::map_row)?;
        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    pub fn upsert(&self, c: &Collection) -> AppResult<()> {
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO collections (id, project_id, name, description, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
               project_id=excluded.project_id, name=excluded.name, description=excluded.description,
               sort_order=excluded.sort_order, updated_at=excluded.updated_at",
            params![c.id, c.project_id, c.name, c.description, c.sort_order, c.created_at, c.updated_at],
        )?;
        Ok(())
    }

    /// 级联删除：先删执行记录 / Request / Folder，再删集合
    pub fn delete(&self, id: &str) -> AppResult<()> {
        let conn = self.db.conn();
        conn.execute(
            "DELETE FROM request_executions WHERE request_id IN (SELECT id FROM requests WHERE collection_id = ?1)",
            params![id],
        )?;
        conn.execute("DELETE FROM requests WHERE collection_id = ?1", params![id])?;
        conn.execute("DELETE FROM folders WHERE collection_id = ?1", params![id])?;
        conn.execute("DELETE FROM collections WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn count_by_project(&self, project_id: &str) -> AppResult<i64> {
        let conn = self.db.conn();
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM collections WHERE project_id = ?1",
            params![project_id],
            |r| r.get(0),
        )?;
        Ok(n)
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Collection> {
        Ok(Collection {
            id: row.get(0)?,
            project_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            sort_order: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }
}
