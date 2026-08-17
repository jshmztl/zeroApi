//! 组合诊断：DNS → TCP → TLS → HTTP（文档 §28）

use std::time::Instant;

use serde::Serialize;

use super::{connect, handshake, resolve, DnsResult, TcpResult, TlsResult};

#[derive(Debug, Clone, Serialize)]
pub struct HttpProbeResult {
    pub ok: bool,
    pub status: Option<u16>,
    pub ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 完整诊断结果（文档 §28）
#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticResult {
    pub target: String,
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub dns: DnsResult,
    pub tcp: TcpResult,
    /// https 目标才有
    pub tls: Option<TlsResult>,
    pub http: Option<HttpProbeResult>,
    pub total_ms: u64,
}

const DEFAULT_PORTS: [(&str, u16); 3] = [("https", 443), ("http", 80), ("wss", 443)];

/// 对目标执行网络诊断
pub async fn diagnose(target: &str) -> DiagnosticResult {
    let start = Instant::now();
    let target = target.trim();

    // 解析 URL
    let (scheme, host, port, path) = parse_target(target);

    // 1. DNS
    let dns = resolve(&host).await;

    // 2. TCP
    let tcp = connect(&host, port).await;

    // 3. TLS（仅 https/wss）
    let tls = if scheme == "https" || scheme == "wss" {
        if tcp.ok {
            Some(handshake(&host, port, &host).await)
        } else {
            // TCP 失败则跳过 TLS
            Some(TlsResult {
                ok: false,
                ms: 0,
                peer_cert_subject: None,
                error: Some("TCP 连接失败，跳过 TLS 探测".to_string()),
            })
        }
    } else {
        None
    };

    // 4. HTTP 探测（GET path，短超时）
    let http = if scheme == "http" || scheme == "https" {
        let probe_start = Instant::now();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .danger_accept_invalid_certs(true)
            .build();
        match client {
            Ok(c) => {
                let url = format!("{}://{}:{}{}", scheme, host, port, path);
                match c.get(&url).send().await {
                    Ok(resp) => Some(HttpProbeResult {
                        ok: true,
                        status: Some(resp.status().as_u16()),
                        ms: probe_start.elapsed().as_millis() as u64,
                        error: None,
                    }),
                    Err(e) => Some(HttpProbeResult {
                        ok: false,
                        status: None,
                        ms: probe_start.elapsed().as_millis() as u64,
                        error: Some(e.to_string()),
                    }),
                }
            }
            Err(e) => Some(HttpProbeResult {
                ok: false,
                status: None,
                ms: 0,
                error: Some(e.to_string()),
            }),
        }
    } else {
        None
    };

    DiagnosticResult {
        target: target.to_string(),
        scheme,
        host,
        port,
        dns,
        tcp,
        tls,
        http,
        total_ms: start.elapsed().as_millis() as u64,
    }
}

/// 解析 target 为 (scheme, host, port, path)
fn parse_target(target: &str) -> (String, String, u16, String) {
    if let Ok(url) = url::Url::parse(target) {
        let scheme = url.scheme().to_string();
        let host = url.host_str().unwrap_or("").to_string();
        let port = url.port().unwrap_or_else(|| {
            DEFAULT_PORTS
                .iter()
                .find(|(s, _)| s == &scheme.as_str())
                .map(|(_, p)| *p)
                .unwrap_or(80)
        });
        let path = if url.path().is_empty() { "/".to_string() } else { url.path().to_string() };
        return (scheme, host, port, path);
    }
    // 无协议：默认 https
    let host = target.to_string();
    ("https".to_string(), host, 443, "/".to_string())
}
