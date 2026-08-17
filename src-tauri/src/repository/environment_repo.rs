//! Environment 持久化

use std::sync::Arc;

use rusqlite::params;

use crate::db::Database;
use crate::domain::{Environment, EnvironmentVariable};
use crate::AppResult;

#[derive(Clone)]
pub struct EnvironmentRepo {
    db: Arc<Database>,
}

impl EnvironmentRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn list(&self) -> AppResult<Vec<Environment>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, base_url, vars, active FROM environments ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i32>(5)?,
            ))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (id, project_id, name, base_url, vars_str, active) = r?;
            let vars: Vec<EnvironmentVariable> =
                serde_json::from_str(&vars_str).unwrap_or_default();
            out.push(Environment {
                id,
                project_id,
                name,
                base_url,
                vars,
                active: active != 0,
            });
        }
        Ok(out)
    }

    pub fn get(&self, id: &str) -> AppResult<Option<Environment>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, base_url, vars, active FROM environments WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i32>(5)?,
            ))
        })?;
        if let Some(r) = rows.next() {
            let (id, project_id, name, base_url, vars_str, active) = r?;
            let vars: Vec<EnvironmentVariable> =
                serde_json::from_str(&vars_str).unwrap_or_default();
            Ok(Some(Environment {
                id,
                project_id,
                name,
                base_url,
                vars,
                active: active != 0,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn upsert(&self, env: &Environment) -> AppResult<()> {
        let conn = self.db.conn();
        let vars = serde_json::to_string(&env.vars)?;
        // active 单选
        if env.active {
            conn.execute("UPDATE environments SET active = 0", [])?;
        }
        conn.execute(
            "INSERT INTO environments (id, project_id, name, base_url, vars, active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
               project_id=excluded.project_id, name=excluded.name, base_url=excluded.base_url,
               vars=excluded.vars, active=excluded.active",
            params![env.id, env.project_id, env.name, env.base_url, vars, env.active as i32],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        let conn = self.db.conn();
        conn.execute("DELETE FROM environments WHERE id = ?1", params![id])?;
        Ok(())
    }
}
