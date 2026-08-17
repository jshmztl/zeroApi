//! Project 命令

use std::path::Path;

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

/// 导出项目到目录（Git-friendly 项目文件格式）
#[tauri::command]
pub fn export_project(state: State<'_, AppState>, project_id: String, dir: String) -> AppResult<()> {
    state.services.project_file.export_project(&project_id, Path::new(&dir))
}

/// 从目录导入项目；返回新项目 id
#[tauri::command]
pub fn import_project(state: State<'_, AppState>, dir: String) -> AppResult<String> {
    state.services.project_file.import_project(Path::new(&dir))
}
