//! Network Diagnostic 命令（文档 §27-29）

use tauri::State;

use crate::network::{
    diagnose, diagnose_proxy, traceroute, DiagnosticResult, ProxyProbeResult, RouteResult,
};
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

/// 通过指定代理对目标执行连通性诊断（代理 TCP 可达 → CONNECT 隧道 + TLS → 经代理 HTTP）
#[tauri::command]
pub async fn diagnose_proxy_network(
    _state: State<'_, AppState>,
    target: String,
    proxy: String,
) -> AppResult<ProxyProbeResult> {
    Ok(diagnose_proxy(&target, &proxy).await)
}

/// 对目标执行逐跳路由追踪（ICMP traceroute）。
/// 阻塞式 ICMP 调用已由 route::traceroute 放入 spawn_blocking，不会阻塞事件循环。
#[tauri::command]
pub async fn diagnose_route(
    _state: State<'_, AppState>,
    target: String,
) -> AppResult<RouteResult> {
    Ok(traceroute(&target).await)
}