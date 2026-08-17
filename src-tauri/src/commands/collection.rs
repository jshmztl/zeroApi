//! Collection / Folder 命令

use tauri::State;

use crate::domain::{Collection, Folder};
use crate::AppResult;
use crate::AppState;

// ---------- Collection ----------

#[tauri::command]
pub fn list_collections(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> AppResult<Vec<Collection>> {
    state.services.collection.list(project_id)
}

#[tauri::command]
pub fn get_collection(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<Option<Collection>> {
    state.services.collection.get(&id)
}

#[tauri::command]
pub fn save_collection(state: State<'_, AppState>, collection: Collection) -> AppResult<String> {
    state.services.collection.save(collection)
}

#[tauri::command]
pub fn delete_collection(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.services.collection.delete(&id)
}

// ---------- Folder ----------

#[tauri::command]
pub fn list_folders(
    state: State<'_, AppState>,
    collection_id: Option<String>,
) -> AppResult<Vec<Folder>> {
    state.services.collection.list_folders(collection_id)
}

#[tauri::command]
pub fn save_folder(state: State<'_, AppState>, folder: Folder) -> AppResult<String> {
    state.services.collection.save_folder(folder)
}

#[tauri::command]
pub fn delete_folder(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.services.collection.delete_folder(&id)
}
