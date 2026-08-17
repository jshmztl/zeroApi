//! Environment 命令

use tauri::State;

use crate::domain::Environment;
use crate::AppResult;
use crate::AppState;

#[tauri::command]
pub fn list_environments(state: State<'_, AppState>) -> AppResult<Vec<Environment>> {
    state.services.environment.list()
}

#[tauri::command]
pub fn get_environment(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<Option<Environment>> {
    state.services.environment.get(&id)
}

#[tauri::command]
pub fn save_environment(state: State<'_, AppState>, env: Environment) -> AppResult<String> {
    state.services.environment.save(env)
}

#[tauri::command]
pub fn delete_environment(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.services.environment.delete(&id)
}
