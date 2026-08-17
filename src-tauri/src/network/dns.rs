//! DNS 解析探测

use std::time::Instant;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DnsResult {
    pub ok: bool,
    /// 解析出的 IP 地址列表
    pub addresses: Vec<String>,
    pub ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 解析主机名（默认端口 0，仅解析）
pub async fn resolve(host: &str) -> DnsResult {
    let start = Instant::now();
    match tokio::net::lookup_host((host, 0)).await {
        Ok(addrs) => {
            let mut addresses: Vec<String> = addrs.map(|a| a.ip().to_string()).collect();
            addresses.sort();
            addresses.dedup();
            let ms = start.elapsed().as_millis() as u64;
            if addresses.is_empty() {
                DnsResult {
                    ok: false,
                    addresses,
                    ms,
                    error: Some("无解析结果".to_string()),
                }
            } else {
                DnsResult {
                    ok: true,
                    addresses,
                    ms,
                    error: None,
                }
            }
        }
        Err(e) => DnsResult {
            ok: false,
            addresses: Vec::new(),
            ms: start.elapsed().as_millis() as u64,
            error: Some(e.to_string()),
        },
    }
}
