//! ZeroApi - Tauri 后端入口（V2）
//!
//! 模块组织（见 ZEROAPI_V2_IMPLEMENTATION.md §2 / §12-16）：
//! - `domain`     领域模型（V2）
//! - `db`         SQLite 连接 + 版本化 Migration
//! - `repository` 持久化层（只负责 SQL）
//! - `transport`  RequestCompiler + HttpTransport + Cookie Session
//! - `service`    业务编排层
//! - `commands`   Tauri IPC 命令（按领域拆分）
//! - `curl`       cURL 命令解析

mod commands;
mod curl;
mod db;
mod domain;
mod error;
mod repository;
mod security;
mod service;
mod transport;

use std::path::PathBuf;
use std::sync::Arc;

use tauri::Manager;

use crate::db::Database;
use crate::repository::Repos;
use crate::service::Services;
use crate::transport::{HttpTransport, SessionManager};

pub use error::{AppError, AppResult};

/// 请求取消注册表（文档 §30）
pub mod cancel {
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::{oneshot, Mutex};

    #[derive(Default)]
    pub struct CancelRegistry(pub Mutex<HashMap<String, oneshot::Sender<()>>>);

    impl CancelRegistry {
        pub fn new() -> Self {
            Self::default()
        }
    }

    pub type SharedRegistry = Arc<CancelRegistry>;
}

/// 共享应用状态
pub struct AppState {
    pub db: Arc<Database>,
    pub services: Services,
    pub transport: Arc<HttpTransport>,
    pub sessions: Arc<SessionManager>,
    pub cancel_registry: cancel::SharedRegistry,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            // 初始化应用数据目录与数据库
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("无法获取 app_data_dir");
            std::fs::create_dir_all(&app_dir)?;
            log::info!("应用数据目录: {:?}", app_dir);

            let db = Arc::new(Database::new(app_dir.join("zeroapi.db"))?);
            db.migrate()?;

            let repos = Repos::new(db.clone());
            let settings = repos.settings.get().unwrap_or_default();

            // HTTP 传输层（不含 Cookie Store，Cookie 按 Project Session 隔离）
            let transport = Arc::new(HttpTransport::new(&settings)?);

            // Cookie Session 管理器（文档 §31）
            let sessions = Arc::new(SessionManager::new());

            // 大响应保存目录
            let responses_dir: PathBuf = app_dir.join("responses");

            let cancel_registry = Arc::new(cancel::CancelRegistry::new());

            let services = Services::new(
                db.clone(),
                &repos,
                transport.clone(),
                sessions.clone(),
                cancel_registry.clone(),
                responses_dir,
            );

            app.manage(AppState {
                db,
                services,
                transport,
                sessions,
                cancel_registry,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Request
            commands::request::send_request,
            commands::request::cancel_request,
            commands::request::save_request,
            commands::request::delete_request,
            commands::request::list_requests,
            commands::request::get_request,
            commands::request::add_favorite,
            commands::request::remove_favorite,
            commands::request::list_favorites,
            // Project
            commands::project::list_projects,
            commands::project::get_project,
            commands::project::save_project,
            commands::project::delete_project,
            commands::project::export_project,
            commands::project::import_project,
            // Collection / Folder
            commands::collection::list_collections,
            commands::collection::get_collection,
            commands::collection::save_collection,
            commands::collection::delete_collection,
            commands::collection::list_folders,
            commands::collection::save_folder,
            commands::collection::delete_folder,
            // Environment
            commands::environment::list_environments,
            commands::environment::get_environment,
            commands::environment::save_environment,
            commands::environment::delete_environment,
            // History
            commands::history::list_history,
            commands::history::get_execution,
            commands::history::delete_history,
            commands::history::clear_history,
            // Import / Export
            commands::import::import_curl,
            commands::import::import_json,
            commands::export::export_json,
            // Settings
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::clear_all_data,
            commands::settings::app_version,
            commands::settings::get_session_cookies,
            commands::settings::clear_session_cookies,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
