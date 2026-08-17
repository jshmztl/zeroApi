//! Environment 持久化（V2 + Secret 安全存储）
//!
//! Secret 变量（kind=secret）：
//! - 明文经 DPAPI 加密后保存到 env_secret_refs 表（SQLite 不落明文）
//! - vars JSON 中只保存引用（secret_ref = 变量名）
//! - 读取时解密注入 value，供编译层使用

use std::sync::Arc;

use rusqlite::params;

use crate::db::Database;
use crate::domain::{Environment, EnvironmentVariable, VariableKind};
use crate::security;
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
            let mut vars: Vec<EnvironmentVariable> =
                serde_json::from_str(&vars_str).unwrap_or_default();
            Self::decrypt_secrets(&conn, &id, &mut vars);
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
            let mut vars: Vec<EnvironmentVariable> =
                serde_json::from_str(&vars_str).unwrap_or_default();
            Self::decrypt_secrets(&conn, &id, &mut vars);
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
        // active 单选
        if env.active {
            conn.execute("UPDATE environments SET active = 0", [])?;
        }

        // 先清除旧 Secret 引用（收集待写入的新引用，环境行落库后再插入）
        conn.execute("DELETE FROM env_secret_refs WHERE env_id = ?1", params![env.id])?;
        let mut new_refs: Vec<(String, String)> = Vec::new(); // (variable_name, encrypted)

        let mut vars = env.vars.clone();
        for v in vars.iter_mut() {
            if v.kind == VariableKind::Secret && !v.value.is_empty() {
                let encrypted = security::encrypt(&v.value).unwrap_or_else(|e| {
                    log::error!("Secret 加密失败({}): {}", v.name, e);
                    v.value.clone()
                });
                if encrypted != v.value {
                    // 加密成功：明文不落库，引用待环境行落库后写入
                    new_refs.push((v.name.clone(), encrypted));
                    v.value.clear();
                    v.secret_ref = Some(v.name.clone());
                }
            } else {
                v.secret_ref = None;
            }
        }

        let vars_json = serde_json::to_string(&vars)?;
        conn.execute(
            "INSERT INTO environments (id, project_id, name, base_url, vars, active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
               project_id=excluded.project_id, name=excluded.name, base_url=excluded.base_url,
               vars=excluded.vars, active=excluded.active",
            params![env.id, env.project_id, env.name, env.base_url, vars_json, env.active as i32],
        )?;

        // 环境行已存在，再写入 Secret 引用（满足外键）
        for (variable_name, encrypted) in new_refs {
            conn.execute(
                "INSERT OR REPLACE INTO env_secret_refs (id, env_id, variable_name, secret_ref, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    uuid::Uuid::new_v4().to_string(),
                    env.id,
                    variable_name,
                    encrypted,
                    chrono::Utc::now().timestamp_millis()
                ],
            )?;
        }
        Ok(())
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        let conn = self.db.conn();
        conn.execute("DELETE FROM env_secret_refs WHERE env_id = ?1", params![id])?;
        conn.execute("DELETE FROM environments WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// 解密注入 Secret 变量（secret_ref 指向 env_secret_refs.variable_name）
    fn decrypt_secrets(conn: &rusqlite::Connection, env_id: &str, vars: &mut [EnvironmentVariable]) {
        for v in vars.iter_mut() {
            if v.kind != VariableKind::Secret {
                v.secret_ref = None;
                continue;
            }
            if v.secret_ref.is_none() && v.value.is_empty() {
                continue;
            }
            let name = v.secret_ref.as_deref().unwrap_or(&v.name);
            let key = v.name.clone();
            let row = conn
                .query_row(
                    "SELECT secret_ref FROM env_secret_refs WHERE env_id = ?1 AND variable_name = ?2",
                    params![env_id, name],
                    |r| r.get::<_, String>(0),
                )
                .ok();
            if let Some(encrypted) = row {
                match security::decrypt(&encrypted) {
                    Ok(plain) => {
                        v.value = plain;
                        v.secret_ref = Some(key);
                    }
                    Err(e) => {
                        log::error!("Secret 解密失败({}/{}): {}", env_id, v.name, e);
                        v.value.clear();
                    }
                }
            }
        }
    }
}
