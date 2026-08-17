//! Settings / 数据维护命令

use tauri::State;

use crate::domain::Settings;
use crate::AppResult;
use crate::AppState;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppResult<Settings> {
    state.services.settings.get()
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> AppResult<()> {
    state.services.settings.save(&settings)
}

/// 清空全部业务数据（保留默认 Project / 未分类集合以维持引用完整性）
#[tauri::command]
pub async fn clear_all_data(state: State<'_, AppState>) -> AppResult<()> {
    {
        let conn = state.db.conn();
        conn.execute_batch(
            "DELETE FROM request_executions;
             DELETE FROM favorites;
             DELETE FROM requests;
             DELETE FROM environments;
             DELETE FROM settings;
             DELETE FROM folders;
             DELETE FROM collections WHERE id != 'uncategorized';",
        )?;
    } // drop 连接锁，避免跨 await 持有非 Send 的 MutexGuard
    state.sessions.clear_all().await;
    Ok(())
}

#[tauri::command]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
