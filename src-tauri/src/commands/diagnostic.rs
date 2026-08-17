//! Network Diagnostic 命令（文档 §27-29）

use tauri::State;

use crate::network::{diagnose, DiagnosticResult};
use crate::AppResult;
use crate::AppState;

/// 对目标执行完整网络诊断（DNS → TCP → TLS → HTTP）
#[tauri::command]
pub async fn diagnose_network(
    _state: State<'_, AppState>,
    target: String,
) -> AppResult<DiagnosticResult> {
    Ok(diagnose(&target).await)
}
