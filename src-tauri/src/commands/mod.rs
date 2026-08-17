//! Tauri IPC 命令（V2 按领域拆分，文档 §15）
//!
//! 组织：Command → Service → Repository → SQLite

pub mod collection;
pub mod environment;
pub mod export;
pub mod history;
pub mod import;
pub mod project;
pub mod request;
pub mod settings;
