//! Import 命令

use serde::Deserialize;
use tauri::State;

use crate::domain::{ImportPayload, Request};
use crate::AppResult;
use crate::AppState;

#[derive(Deserialize)]
pub struct ImportArgs {
    pub content: String,
}

/// 导入 cURL 命令
#[tauri::command]
pub fn import_curl(state: State<'_, AppState>, command: String) -> AppResult<Request> {
    state.services.import.import_curl(&command)
}

/// 导入 JSON 备份
#[tauri::command]
pub fn import_json(state: State<'_, AppState>, args: ImportArgs) -> AppResult<ImportPayload> {
    state.services.import.import_json(&args.content)
}
