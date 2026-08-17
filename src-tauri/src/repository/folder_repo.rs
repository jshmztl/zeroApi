//! Folder 持久化

use std::sync::Arc;

use rusqlite::params;

use crate::db::Database;
use crate::domain::Folder;
use crate::AppResult;

#[derive(Clone)]
pub struct FolderRepo {
    db: Arc<Database>,
}

impl FolderRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn list(&self, collection_id: Option<&str>) -> AppResult<Vec<Folder>> {
        let conn = self.db.conn();
        let mut out = Vec::new();
        match collection_id {
            Some(cid) => {
                let mut stmt = conn.prepare(
                    "SELECT id, collection_id, parent_id, name, sort_order
                     FROM folders WHERE collection_id = ?1 ORDER BY sort_order ASC, name ASC",
                )?;
                let rows = stmt.query_map(params![cid], Self::map_row)?;
                for r in rows {
                    out.push(r?);
                }
            }
            None => {
                let mut stmt = conn.prepare(
                    "SELECT id, collection_id, parent_id, name, sort_order
                     FROM folders ORDER BY sort_order ASC, name ASC",
                )?;
                let rows = stmt.query_map([], Self::map_row)?;
                for r in rows {
                    out.push(r?);
                }
            }
        }
        Ok(out)
    }

    pub fn get(&self, id: &str) -> AppResult<Option<Folder>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, collection_id, parent_id, name, sort_order FROM folders WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::map_row)?;
        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    pub fn upsert(&self, f: &Folder) -> AppResult<()> {
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO folders (id, collection_id, parent_id, name, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
               collection_id=excluded.collection_id, parent_id=excluded.parent_id,
               name=excluded.name, sort_order=excluded.sort_order",
            params![f.id, f.collection_id, f.parent_id, f.name, f.sort_order],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        let conn = self.db.conn();
        // 子级 Folder / Request 一并清理
        conn.execute("DELETE FROM requests WHERE folder_id = ?1", params![id])?;
        conn.execute("DELETE FROM folders WHERE parent_id = ?1", params![id])?;
        conn.execute("DELETE FROM folders WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Folder> {
        Ok(Folder {
            id: row.get(0)?,
            collection_id: row.get(1)?,
            parent_id: row.get(2)?,
            name: row.get(3)?,
            sort_order: row.get(4)?,
        })
    }
}
