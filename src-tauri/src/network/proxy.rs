//! Proxy 探针（文档 §29 第二版）
//!
//! 通过代理探测目标连通性，阶段：
//!   1. proxy_tcp — 代理本身 TCP 可达
//!   2. tls       — 通过代理建立 CONNECT 隧道 + TLS 握手（https/wss 目标）
//!   3. http      — 通过代理完成一次 HTTP 请求
//!
//! 语义说明：HTTP/HTTPS 代理通常是「客户端把 host:port 交给代理，由代理侧做 DNS」；
//! 因此 `dns` 字段为本地解析结果，仅作参考，真正的目标解析发生在代理侧。

use std::time::Instant;

use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use super::diagnostic::{parse_target, HttpProbeResult};
use super::dns::resolve;
use super::tcp::connect;
use super::tls::TlsResult;
use super::tls::{build_connector, x509_parser_subject};

#[derive(Debug, Clone, Serialize)]
pub struct ProxyProbeResult {
    pub proxy_host: String,
    pub proxy_port: u16,
    /// 代理本身 TCP 可达性
    pub proxy_tcp: super::tcp::TcpResult,
    pub target: String,
    pub scheme: String,
    pub target_host: String,
    /// 目标本地 DNS（HTTP 代理的真实 DNS 在代理侧完成，仅供对照）
    pub dns: super::dns::DnsResult,
    /// 通过代理的 TLS 握手（https/wss 目标）
    pub tls: Option<TlsResult>,
    /// 通过代理的 HTTP 探测
    pub http: Option<HttpProbeResult>,
    pub total_ms: u64,
}

const PROXY_DEFAULT_PORTS: [(&str, u16); 4] = [("http", 80), ("https", 443), ("socks5", 1080), ("socks5h", 1080)];
const PROXY_RESPONSE_OK: &str = "200";

/// 通过指定代理对目标执行连通性探测
pub async fn diagnose_proxy(target: &str, proxy_url: &str) -> ProxyProbeResult {
    let start = Instant::now();
    let proxy_url = proxy_url.trim();
    let target = target.trim();

    // 解析目标
    let (scheme, target_host, target_port, path) = parse_target(target);
    let dns = resolve(&target_host).await;

    // 解析代理 URL；缺协议时按 http 处理
    let normalized = parse_proxy_url(proxy_url);

    let (proxy_host, proxy_port, proxy_scheme) = match normalized {
        Some((h, p, s)) => (h, p, s),
        None => {
            // 代理 URL 非法：返回带错误标识的结果
            return ProxyProbeResult {
                proxy_host: proxy_url.to_string(),
                proxy_port: 0,
                proxy_tcp: super::tcp::TcpResult {
                    ok: false,
                    port: 0,
                    ms: 0,
                    error: Some(format!("无效的代理地址: {}", proxy_url)),
                },
                target: target.to_string(),
                scheme,
                target_host,
                dns,
                tls: None,
                http: None,
                total_ms: start.elapsed().as_millis() as u64,
            };
        }
    };

    // 1. 代理 TCP 可达
    let proxy_tcp = connect(&proxy_host, proxy_port).await;

    // 2. 通过代理的 TLS 握手（https/wss 目标）
    let tls = if (scheme == "https" || scheme == "wss") && proxy_tcp.ok {
        Some(tls_through_proxy(&proxy_host, proxy_port, &target_host, target_port, &target_host).await)
    } else if scheme == "https" || scheme == "wss" {
        Some(TlsResult {
            ok: false,
            ms: 0,
            peer_cert_subject: None,
            error: Some("代理不可达，跳过 TLS 探测".into()),
        })
    } else {
        None
    };

    // 3. 通过代理的 HTTP 探测
    let http = if scheme == "http" || scheme == "https" {
        let probe_start = Instant::now();
        let proxy_result = reqwest::Proxy::all(format!("{}://{}:{}", proxy_scheme, proxy_host, proxy_port));
        let client = match proxy_result {
            Ok(proxy) => reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(8))
                .danger_accept_invalid_certs(true)
                .proxy(proxy)
                .build(),
            Err(e) => Err(e),
        };
        match client {
            Ok(c) => {
                let url = format!("{}://{}:{}{}", scheme, target_host, target_port, path);
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
                ms: probe_start.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
            }),
        }
    } else {
        None
    };

    ProxyProbeResult {
        proxy_host,
        proxy_port,
        proxy_tcp,
        target: target.to_string(),
        scheme,
        target_host,
        dns,
        tls,
        http,
        total_ms: start.elapsed().as_millis() as u64,
    }
}

/// 通过 HTTP 代理隧道完成 TLS 握手（CONNECT → 200 → TLS）
async fn tls_through_proxy(
    proxy_host: &str,
    proxy_port: u16,
    target_host: &str,
    target_port: u16,
    server_name: &str,
) -> TlsResult {
    let start = Instant::now();
    let connector = match build_connector() {
        Ok(c) => c,
        Err(e) => return tls_fail(&start, Some(e)),
    };

    let connect_line = format!("CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\nProxy-Connection: Keep-Alive\r\n\r\n", target_host, target_port, target_host, target_port);

    let mut stream = match TcpStream::connect((proxy_host, proxy_port)).await {
        Ok(s) => s,
        Err(e) => return tls_fail(&start, Some(format!("连接代理失败: {}", e))),
    };

    if let Err(e) = stream.write_all(connect_line.as_bytes()).await {
        return tls_fail(&start, Some(format!("发送 CONNECT 失败: {}", e)));
    }
    // 逐字节读取响应头，直到空行；避免吞掉隧道中的 TLS 握手字节
    if !read_connect_response_ok(&mut stream).await {
        return tls_fail(&start, Some("代理未返回 200（CONNECT 被拒绝）".into()));
    }

    let name = match rustls::pki_types::ServerName::try_from(server_name.to_string()) {
        Ok(n) => n,
        Err(e) => return tls_fail(&start, Some(format!("无效的服务器名称: {}", e))),
    };

    match connector.connect(name, stream).await {
        Ok(tls) => {
            let peer_cert_subject = tls
                .get_ref()
                .1
                .peer_certificates()
                .and_then(|certs| certs.first())
                .map(|c| c.as_ref())
                .and_then(|der| x509_parser_subject(der).or_else(|| Some("(证书主题不可解析)".to_string())));
            TlsResult {
                ok: true,
                ms: start.elapsed().as_millis() as u64,
                peer_cert_subject,
                error: None,
            }
        }
        Err(e) => tls_fail(&start, Some(e.to_string())),
    }
}

/// 读取代理 CONNECT 响应头，判断是否 200（逐字节读，不吞隧道数据）
async fn read_connect_response_ok(stream: &mut TcpStream) -> bool {
    let mut buf = Vec::with_capacity(256);
    let mut one = [0u8; 1];
    // 最多读取 8KB，防止代理挂起
    for _ in 0..8192 {
        match stream.read_exact(&mut one).await {
            Ok(_) => {
                buf.push(one[0]);
                if buf.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            Err(_) => return false,
        }
    }
    let head = String::from_utf8_lossy(&buf);
    let first_line = head.lines().next().unwrap_or("");
    first_line
        .split_whitespace()
        .nth(1)
        .map(|code| code == PROXY_RESPONSE_OK)
        .unwrap_or(false)
}

fn tls_fail(start: &Instant, error: Option<String>) -> TlsResult {
    TlsResult {
        ok: false,
        ms: start.elapsed().as_millis() as u64,
        peer_cert_subject: None,
        error,
    }
}

/// 解析代理 URL 为 (host, port, scheme)；缺协议按 http，返回 None 表示非法
fn parse_proxy_url(raw: &str) -> Option<(String, u16, String)> {
    let with_scheme = if raw.contains("://") { raw.to_string() } else { format!("http://{}", raw) };
    let url = url::Url::parse(&with_scheme).ok()?;
    let scheme = url.scheme().to_lowercase();
    if !matches!(scheme.as_str(), "http" | "https" | "socks5" | "socks5h") {
        return None;
    }
    let host = url.host_str()?.to_string();
    let port = url
        .port()
        .unwrap_or_else(|| PROXY_DEFAULT_PORTS.iter().find(|(s, _)| s == &scheme.as_str()).map(|(_, p)| *p).unwrap_or(8080));
    Some((host, port, scheme))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_http_proxy() {
        let (h, p, s) = parse_proxy_url("http://127.0.0.1:7890").unwrap();
        assert_eq!(h, "127.0.0.1");
        assert_eq!(p, 7890);
        assert_eq!(s, "http");
    }

    #[test]
    fn parse_proxy_without_scheme_defaults_http() {
        let (h, p, s) = parse_proxy_url("127.0.0.1:8080").unwrap();
        assert_eq!(h, "127.0.0.1");
        assert_eq!(p, 8080);
        assert_eq!(s, "http");
    }

    #[test]
    fn parse_socks_default_port() {
        let (_, p, s) = parse_proxy_url("socks5://proxy.local").unwrap();
        assert_eq!(p, 1080);
        assert_eq!(s, "socks5");
    }

    #[test]
    fn parse_proxy_scheme_default_port() {
        let (_, p, _) = parse_proxy_url("https://10.0.0.1").unwrap();
        assert_eq!(p, 443);
    }

    #[test]
    fn invalid_proxy_returns_none() {
        assert!(parse_proxy_url("ftp://127.0.0.1:21").is_none());
        assert!(parse_proxy_url("not a url with spaces").is_none());
    }
}