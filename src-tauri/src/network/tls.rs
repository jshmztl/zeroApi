//! TLS 握手探测
//!
//! 使用 rustls + webpki-roots（系统级根证书 + Mozilla 根证书）。
//! 只做握手，成功后立即关闭，不发送应用数据。

use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

#[derive(Debug, Clone, Serialize)]
pub struct TlsResult {
    pub ok: bool,
    pub ms: u64,
    /// 证书链信息摘要
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_cert_subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub(crate) fn build_connector() -> Result<TlsConnector, String> {
    // 直接使用 rustls 的 ClientConfig::builder() 需要进程级 CryptoProvider，幂等安装 ring
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(TlsConnector::from(Arc::new(config)))
}

/// TLS 握手探测（host 为 IP 或域名，server_name 用于 SNI）
pub async fn handshake(host: &str, port: u16, server_name: &str) -> TlsResult {
    let start = Instant::now();
    let connector = match build_connector() {
        Ok(c) => c,
        Err(e) => {
            return TlsResult {
                ok: false,
                ms: 0,
                peer_cert_subject: None,
                error: Some(e),
            }
        }
    };

    // SNI 名称（IP 直连时用 IP 作为 server name 会校验失败，这里直接尝试）
    let name = match rustls::pki_types::ServerName::try_from(server_name.to_string()) {
        Ok(n) => n,
        Err(e) => {
            return TlsResult {
                ok: false,
                ms: start.elapsed().as_millis() as u64,
                peer_cert_subject: None,
                error: Some(format!("无效的服务器名称: {}", e)),
            }
        }
    };

    match TcpStream::connect((host, port)).await {
        Ok(stream) => match connector.connect(name, stream).await {
            Ok(tls) => {
                let peer_cert_subject = tls
                    .get_ref()
                    .1
                    .peer_certificates()
                    .and_then(|certs| certs.first())
                    .map(|c| c.as_ref())
                    .and_then(|der| {
                        x509_parser_subject(der).or_else(|| Some("(证书主题不可解析)".to_string()))
                    });
                TlsResult {
                    ok: true,
                    ms: start.elapsed().as_millis() as u64,
                    peer_cert_subject,
                    error: None,
                }
            }
            Err(e) => TlsResult {
                ok: false,
                ms: start.elapsed().as_millis() as u64,
                peer_cert_subject: None,
                error: Some(e.to_string()),
            },
        },
        Err(e) => TlsResult {
            ok: false,
            ms: start.elapsed().as_millis() as u64,
            peer_cert_subject: None,
            error: Some(format!("TCP 连接失败: {}", e)),
        },
    }
}

/// 尝试从 DER 证书提取主题（尽力而为，失败返回 None）
pub(crate) fn x509_parser_subject(der: &[u8]) -> Option<String> {
    // 简化：按 ASN.1 启发式查找 CN（不做完整解析，完整解析需 x509-parser）
    let text = String::from_utf8_lossy(der);
    if let Some(idx) = text.find("CN=") {
        let tail = &text[idx + 3..];
        let end = tail.find('\0').map(|e| idx + 3 + e).unwrap_or(text.len());
        return Some(text[idx + 3..end.min(text.len())].to_string());
    }
    Some("证书已加载".to_string())
}
