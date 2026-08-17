//! Settings 持久化

use std::sync::Arc;

use rusqlite::params;

use crate::db::Database;
use crate::domain::Settings;
use crate::AppResult;

#[derive(Clone)]
pub struct SettingsRepo {
    db: Arc<Database>,
}

impl SettingsRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn get(&self) -> AppResult<Settings> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'app'")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let raw: String = row.get(0)?;
            Ok(serde_json::from_str(&raw).unwrap_or_default())
        } else {
            Ok(Settings::default())
        }
    }

    pub fn save(&self, settings: &Settings) -> AppResult<()> {
        let conn = self.db.conn();
        let raw = serde_json::to_string(settings)?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('app', ?1)",
            params![raw],
        )?;
        Ok(())
    }
}
