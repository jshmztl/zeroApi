//! Project 命令

use tauri::State;

use crate::domain::Project;
use crate::AppResult;
use crate::AppState;

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> AppResult<Vec<Project>> {
    state.services.project.list()
}

#[tauri::command]
pub fn get_project(state: State<'_, AppState>, id: String) -> AppResult<Option<Project>> {
    state.services.project.get(&id)
}

#[tauri::command]
pub fn save_project(state: State<'_, AppState>, project: Project) -> AppResult<String> {
    state.services.project.save(project)
}

#[tauri::command]
pub fn delete_project(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.services.project.delete(&id)
}
