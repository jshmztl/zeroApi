//! SQLite 持久化层（V2）
//!
//! - 版本化 Migration：schema 由 `migrations/*.sql` 定义，旧数据转换由 Rust 数据迁移完成
//! - 仅负责连接管理与迁移执行；具体 CRUD 在 `repository/` 层
//!
//! 迁移设计（见 ZEROAPI_V2_IMPLEMENTATION.md §9 / §18）：
//! - 每个迁移在事务内执行（原子），版本记录在 `schema_migrations` 表
//! - 不修改旧迁移，每次 Schema 变化新增 Migration

mod migrations;

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::AppResult;

pub(crate) use migrations::UNCATEGORIZED_COLLECTION;

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

    /// 执行版本化迁移（启动时调用一次）
    pub fn migrate(&self) -> AppResult<()> {
        // Mutex 中毒仅表示前一次持锁期间发生 panic；SQLite 事务原子性保证，
        // 通过 into_inner 取回底层连接仍可继续使用，避免此处直接 panic。
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        migrations::run(&conn)
    }

    /// 访问底层连接（repository 层使用）
    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }
}
