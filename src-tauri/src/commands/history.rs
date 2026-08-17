//! History 命令（基于 RequestExecution）

use tauri::State;

use crate::domain::{ExecutionListItem, RequestExecution};
use crate::AppResult;
use crate::AppState;

#[tauri::command]
pub fn list_history(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> AppResult<Vec<ExecutionListItem>> {
    state.services.history.list(limit.unwrap_or(100))
}

#[tauri::command]
pub fn get_execution(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<Option<RequestExecution>> {
    state.services.history.get(&id)
}

#[tauri::command]
pub fn delete_history(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.services.history.delete(&id)
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> AppResult<()> {
    state.services.history.clear()
}
