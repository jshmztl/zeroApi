//! TCP 连接探测

use std::time::Instant;

use serde::Serialize;
use tokio::net::TcpStream;

#[derive(Debug, Clone, Serialize)]
pub struct TcpResult {
    pub ok: bool,
    pub port: u16,
    pub ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 尝试建立 TCP 连接（成功后立即关闭）
pub async fn connect(host: &str, port: u16) -> TcpResult {
    let start = Instant::now();
    match TcpStream::connect((host, port)).await {
        Ok(_stream) => TcpResult {
            ok: true,
            port,
            ms: start.elapsed().as_millis() as u64,
            error: None,
        },
        Err(e) => TcpResult {
            ok: false,
            port,
            ms: start.elapsed().as_millis() as u64,
            error: Some(e.to_string()),
        },
    }
}
