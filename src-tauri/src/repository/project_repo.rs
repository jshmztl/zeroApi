//! Project 持久化

use std::sync::Arc;

use rusqlite::params;

use crate::db::Database;
use crate::domain::Project;
use crate::AppResult;

#[derive(Clone)]
pub struct ProjectRepo {
    db: Arc<Database>,
}

impl ProjectRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn list(&self) -> AppResult<Vec<Project>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, root_path, created_at, updated_at FROM projects ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (id, name, description, root_path, created_at, updated_at) = r?;
            out.push(Project {
                id,
                name,
                description,
                root_path,
                created_at,
                updated_at,
            });
        }
        Ok(out)
    }

    pub fn get(&self, id: &str) -> AppResult<Option<Project>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, root_path, created_at, updated_at FROM projects WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })?;
        if let Some(r) = rows.next() {
            let (id, name, description, root_path, created_at, updated_at) = r?;
            Ok(Some(Project {
                id,
                name,
                description,
                root_path,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn upsert(&self, p: &Project) -> AppResult<()> {
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO projects (id, name, description, root_path, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
               name=excluded.name, description=excluded.description,
               root_path=excluded.root_path, updated_at=excluded.updated_at",
            params![p.id, p.name, p.description, p.root_path, p.created_at, p.updated_at],
        )?;
        Ok(())
    }

    /// 级联删除项目：先删该项目所有集合下的执行记录 / Request / Folder / Collection
    pub fn delete(&self, id: &str) -> AppResult<()> {
        let conn = self.db.conn();
        conn.execute(
            "DELETE FROM request_executions WHERE request_id IN (
                SELECT id FROM requests WHERE collection_id IN (SELECT id FROM collections WHERE project_id = ?1)
            )",
            params![id],
        )?;
        conn.execute(
            "DELETE FROM requests WHERE collection_id IN (SELECT id FROM collections WHERE project_id = ?1)",
            params![id],
        )?;
        conn.execute(
            "DELETE FROM folders WHERE collection_id IN (SELECT id FROM collections WHERE project_id = ?1)",
            params![id],
        )?;
        conn.execute("DELETE FROM collections WHERE project_id = ?1", params![id])?;
        conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(())
    }
}
