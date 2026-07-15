//! ZeroApi - Tauri 后端入口
//!
//! 模块组织：
//! - `models`  数据结构定义
//! - `db`      SQLite 持久化
//! - `http`    reqwest HTTP 客户端
//! - `curl`    cURL 命令解析
//! - `commands` Tauri IPC 命令

mod commands;
mod curl;
mod db;
mod error;
mod http;
mod models;

use std::sync::Arc;
use tauri::Manager;

use crate::db::Database;

pub use error::{AppError, AppResult};

/// 请求取消注册表
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
    pub http_client: Arc<reqwest::Client>,
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

            let db = Database::new(app_dir.join("zeroapi.db"))?;
            db.migrate()?;

            // 全局 HTTP 客户端
            let http_client = reqwest::Client::builder()
                .user_agent("ZeroApi/1.0")
                .cookie_store(true)
                .danger_accept_invalid_certs(false)
                .build()
                .expect("初始化 HTTP 客户端失败");

            app.manage(AppState {
                db: Arc::new(db),
                http_client: Arc::new(http_client),
                cancel_registry: Arc::new(cancel::CancelRegistry::new()),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::send_request,
            commands::cancel_request,
            commands::save_request,
            commands::delete_request,
            commands::list_history,
            commands::clear_history,
            commands::list_favorites,
            commands::add_favorite,
            commands::remove_favorite,
            commands::list_collections,
            commands::save_collection,
            commands::update_collection,
            commands::delete_collection,
            commands::save_saved_request,
            commands::list_saved_requests,
            commands::list_environments,
            commands::save_environment,
            commands::delete_environment,
            commands::import_curl,
            commands::export_json,
            commands::import_json,
            commands::get_settings,
            commands::save_settings,
            commands::clear_all_data,
            commands::app_version,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
