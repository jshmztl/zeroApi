//! Import 命令

use serde::Deserialize;
use tauri::State;

use crate::domain::{ImportPayload, Request};
use crate::service::OpenApiImportResult;
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

/// 导入 OpenAPI 3.0/3.1 文档（YAML / JSON）
#[tauri::command]
pub fn import_openapi(
    state: State<'_, AppState>,
    content: String,
    project_id: Option<String>,
) -> AppResult<OpenApiImportResult> {
    state.services.openapi.import(&content, project_id)
}

/// 从 URL 抓取并导入 OpenAPI 文档
#[tauri::command]
pub async fn import_openapi_url(
    state: State<'_, AppState>,
    url: String,
    project_id: Option<String>,
) -> AppResult<OpenApiImportResult> {
    state.services.openapi.import_url(&url, project_id).await
}
