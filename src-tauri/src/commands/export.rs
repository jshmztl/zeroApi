//! Export 命令

use tauri::State;

use crate::AppResult;
use crate::AppState;

/// 导出 JSON 备份（不含 Secret 明文 / 历史 / Cookie / 设置）
#[tauri::command]
pub fn export_json(state: State<'_, AppState>) -> AppResult<String> {
    state.services.export.export()
}
