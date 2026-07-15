//! Tauri IPC 命令(前端调用入口)
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::http::{collect_env_vars, execute};
use crate::models::*;
use crate::AppError;
use crate::AppResult;
use crate::AppState;

#[tauri::command]
pub async fn send_request(
    state: State<'_, AppState>,
    request: Request,
    client_id: Option<String>,
) -> AppResult<ResponseSnapshot> {
    let cid = client_id.unwrap_or_else(|| Uuid::new_v4().to_string());

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    state.cancel_registry.0.lock().await.insert(cid.clone(), tx);

    let db = state.db.clone();
    let http_client = state.http_client.clone();
    let cancel_registry = state.cancel_registry.clone();

    let result = async {
        let settings = db.get_settings().unwrap_or_default();
        let envs = db.list_environments()?;
        let active = envs.iter().find(|e| e.active);
        let env_vars = collect_env_vars(active);

        let resp = execute(&http_client, &request, &env_vars, &settings).await?;

        // 自动保存历史
        if settings.auto_save_history {
            let item = HistoryItem {
                id: Uuid::new_v4().to_string(),
                request,
                response: resp.clone(),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            db.add_history(&item)?;
        }

        Ok::<_, AppError>(resp)
    };

    let resp = tokio::select! {
        r = result => r?,
        _ = rx => return Err(AppError::Other("请求已取消".into())),
    };

    cancel_registry.0.lock().await.remove(&cid);
    Ok(resp)
}

#[tauri::command]
pub async fn cancel_request(
    state: State<'_, AppState>,
    client_id: String,
) -> AppResult<bool> {
    if let Some(tx) = state.cancel_registry.0.lock().await.remove(&client_id) {
        let _ = tx.send(());
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn save_request(state: State<'_, AppState>, mut request: Request) -> AppResult<String> {
    if request.id.is_empty() {
        request.id = Uuid::new_v4().to_string();
    }
    let fav = Favorite {
        id: Uuid::new_v4().to_string(),
        request,
        created_at: chrono::Utc::now().timestamp_millis(),
    };
    state.db.add_favorite(&fav)?;
    Ok(fav.id)
}

#[tauri::command]
pub async fn delete_request(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.db.remove_favorite(&id)
}

#[tauri::command]
pub async fn list_history(state: State<'_, AppState>, limit: Option<u32>) -> AppResult<Vec<HistoryItem>> {
    let l = limit.unwrap_or(100);
    state.db.list_history(l)
}

#[tauri::command]
pub async fn clear_history(state: State<'_, AppState>) -> AppResult<()> {
    state.db.clear_history()
}

#[tauri::command]
pub async fn delete_history(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.db.delete_history(&id)
}

#[tauri::command]
pub async fn list_favorites(state: State<'_, AppState>) -> AppResult<Vec<Favorite>> {
    state.db.list_favorites()
}

#[tauri::command]
pub async fn add_favorite(state: State<'_, AppState>, mut request: Request) -> AppResult<String> {
    if request.id.is_empty() {
        request.id = Uuid::new_v4().to_string();
    }
    let fav = Favorite {
        id: Uuid::new_v4().to_string(),
        request,
        created_at: chrono::Utc::now().timestamp_millis(),
    };
    state.db.add_favorite(&fav)?;
    Ok(fav.id)
}

#[tauri::command]
pub async fn remove_favorite(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.db.remove_favorite(&id)
}

#[tauri::command]
pub async fn list_collections(state: State<'_, AppState>) -> AppResult<Vec<Collection>> {
    state.db.list_collections()
}

#[tauri::command]
pub async fn save_collection(state: State<'_, AppState>, mut collection: Collection) -> AppResult<String> {
    if collection.id.is_empty() {
        collection.id = Uuid::new_v4().to_string();
    }
    if collection.created_at == 0 {
        collection.created_at = chrono::Utc::now().timestamp_millis();
    }
    collection.updated_at = chrono::Utc::now().timestamp_millis();
    state.db.upsert_collection(&collection)?;
    Ok(collection.id)
}

#[tauri::command]
pub async fn update_collection(
    state: State<'_, AppState>,
    id: String,
    patch: CollectionPatch,
) -> AppResult<()> {
    state.db.update_collection(&id, &patch)
}

#[tauri::command]
pub async fn save_saved_request(
    state: State<'_, AppState>,
    mut request: Request,
    collection_id: Option<String>,
) -> AppResult<String> {
    if request.id.is_empty() {
        request.id = Uuid::new_v4().to_string();
    }
    let cid = collection_id.as_deref();
    state.db.upsert_saved_request(&request, cid)?;
    Ok(request.id)
}

#[tauri::command]
pub async fn list_saved_requests(
    state: State<'_, AppState>,
    collection_id: Option<String>,
) -> AppResult<Vec<Request>> {
    state.db.list_saved_requests(collection_id.as_deref())
}

#[tauri::command]
pub async fn delete_collection(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.db.delete_collection(&id)
}

#[tauri::command]
pub async fn list_environments(state: State<'_, AppState>) -> AppResult<Vec<Environment>> {
    state.db.list_environments()
}

#[tauri::command]
pub async fn save_environment(state: State<'_, AppState>, mut env: Environment) -> AppResult<String> {
    if env.id.is_empty() {
        env.id = Uuid::new_v4().to_string();
    }
    state.db.upsert_environment(&env)?;
    Ok(env.id)
}

#[tauri::command]
pub async fn delete_environment(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.db.delete_environment(&id)
}

#[tauri::command]
pub fn import_curl(command: String) -> AppResult<Request> {
    crate::curl::parse(&command)
}

#[derive(Deserialize)]
pub struct ImportArgs {
    pub content: String,
}

#[tauri::command]
pub async fn import_json(state: State<'_, AppState>, args: ImportArgs) -> AppResult<ImportPayload> {
    let payload: ExportPayload = serde_json::from_str(&args.content)
        .map_err(|e| AppError::Other(format!("JSON 解析失败: {}", e)))?;
    let out = ImportPayload {
        requests: payload.requests.clone(),
        collections: payload.collections.clone(),
        environments: payload.environments.clone(),
        favorites: payload.favorites.clone(),
    };
    for e in payload.environments {
        state.db.upsert_environment(&e)?;
    }
    for c in payload.collections {
        state.db.upsert_collection(&c)?;
    }
    for f in payload.favorites {
        state.db.add_favorite(&f)?;
    }
    Ok(out)
}

#[tauri::command]
pub async fn export_json(
    state: State<'_, AppState>,
    collections: Option<Vec<Collection>>,
    environments: Option<Vec<Environment>>,
) -> AppResult<String> {
    let favs = state.db.list_favorites()?;
    let payload = ExportPayload {
        version: 1,
        requests: favs.iter().map(|f| f.request.clone()).collect(),
        collections: collections.unwrap_or_default(),
        environments: environments.unwrap_or_default(),
        favorites: favs,
    };
    Ok(serde_json::to_string_pretty(&payload)?)
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> AppResult<Settings> {
    state.db.get_settings()
}

#[tauri::command]
pub async fn save_settings(state: State<'_, AppState>, settings: Settings) -> AppResult<()> {
    state.db.save_settings(&settings)
}

#[tauri::command]
pub async fn clear_all_data(state: State<'_, AppState>) -> AppResult<()> {
    state.db.clear_all()
}

#[tauri::command]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
