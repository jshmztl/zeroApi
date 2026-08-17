//! Request 命令

use tauri::State;

use crate::domain::*;
use crate::AppResult;
use crate::AppState;

/// 发送请求（可取消）
#[tauri::command]
pub async fn send_request(
    state: State<'_, AppState>,
    request: Request,
    client_id: Option<String>,
) -> AppResult<ResponseSnapshot> {
    state.services.request.send(&request, client_id).await
}

/// 取消请求：真正终止底层 HTTP Future（文档 §30）
#[tauri::command]
pub async fn cancel_request(state: State<'_, AppState>, client_id: String) -> AppResult<bool> {
    if let Some(tx) = state
        .cancel_registry
        .0
        .lock()
        .await
        .remove(&client_id)
    {
        let _ = tx.send(());
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 保存请求（upsert）
#[tauri::command]
pub fn save_request(state: State<'_, AppState>, request: Request) -> AppResult<String> {
    state.services.request.save_request(request)
}

#[tauri::command]
pub fn delete_request(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.services.request.delete_request(&id)
}

#[tauri::command]
pub fn list_requests(
    state: State<'_, AppState>,
    collection_id: Option<String>,
    folder_id: Option<String>,
) -> AppResult<Vec<Request>> {
    state.services.request.list_requests(collection_id, folder_id)
}

#[tauri::command]
pub fn get_request(state: State<'_, AppState>, id: String) -> AppResult<Option<Request>> {
    state.services.request.get_request(&id)
}

// ---------- 收藏 ----------

#[tauri::command]
pub fn add_favorite(state: State<'_, AppState>, request: Request) -> AppResult<String> {
    state.services.request.add_favorite(request)
}

#[tauri::command]
pub fn remove_favorite(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.services.request.remove_favorite(&id)
}

#[tauri::command]
pub fn list_favorites(state: State<'_, AppState>) -> AppResult<Vec<Favorite>> {
    state.services.request.list_favorites()
}

// ---------- cURL 导出（文档 §24） ----------

/// 把 Request 转成 cURL 命令
#[tauri::command]
pub fn export_curl(request: Request) -> AppResult<String> {
    Ok(crate::curl::to_curl(&request))
}
