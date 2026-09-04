//! TLS 握手探测 + 证书链诊断
//!
//! 使用 rustls + webpki-roots（系统级根证书 + Mozilla 根证书）。
//! 只做握手，成功后立即关闭，不发送应用数据。
//! 握手成功后用 x509-parser 解析对端返回的完整证书链（服务端 + 中间证书）。

use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use x509_parser::prelude::{parse_x509_certificate, X509Name};

use rustls::pki_types::CertificateDer;

/// 单个证书的链内信息
#[derive(Debug, Clone, Serialize)]
pub struct CertInfo {
    /// 链内深度（0 = 叶证书 / 服务端证书）
    pub depth: usize,
    pub subject: String,
    pub issuer: String,
    pub serial: String,
    /// 有效期（YYYY-MM-DD）
    pub not_before: String,
    pub not_after: String,
    pub is_ca: bool,
    /// 是否已过期（now > not_after）
    pub expired: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TlsResult {
    pub ok: bool,
    pub ms: u64,
    /// 证书链信息摘要
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_cert_subject: Option<String>,
    /// 完整证书链（握手成功时提供）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chain: Vec<CertInfo>,
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

/// TLS 握手探测（host 为 IP 或域名，server_name 用于 SNI）。
/// 成功后同时解析并返回对端证书链。
pub async fn handshake(host: &str, port: u16, server_name: &str) -> TlsResult {
    let start = Instant::now();
    let connector = match build_connector() {
        Ok(c) => c,
        Err(e) => {
            return TlsResult {
                ok: false,
                ms: 0,
                peer_cert_subject: None,
                chain: Vec::new(),
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
                chain: Vec::new(),
                error: Some(format!("无效的服务器名称: {}", e)),
            }
        }
    };

    match TcpStream::connect((host, port)).await {
        Ok(stream) => match connector.connect(name, stream).await {
            Ok(tls) => {
                let chain = tls
                    .get_ref()
                    .1
                    .peer_certificates()
                    .map(|certs| build_chain_info(certs))
                    .unwrap_or_default();
                let peer_cert_subject = chain.first().map(|c| c.subject.clone());
                TlsResult {
                    ok: true,
                    ms: start.elapsed().as_millis() as u64,
                    peer_cert_subject,
                    chain,
                    error: None,
                }
            }
            Err(e) => TlsResult {
                ok: false,
                ms: start.elapsed().as_millis() as u64,
                peer_cert_subject: None,
                chain: Vec::new(),
                error: Some(e.to_string()),
            },
        },
        Err(e) => TlsResult {
            ok: false,
            ms: start.elapsed().as_millis() as u64,
            peer_cert_subject: None,
            chain: Vec::new(),
            error: Some(format!("TCP 连接失败: {}", e)),
        },
    }
}

/// 解析对端证书链（DER 序列）为结构化摘要。
/// 解析失败的单张证书用占位信息体现，不中断整体。
pub(crate) fn build_chain_info(certs: &[CertificateDer<'_>]) -> Vec<CertInfo> {
    let now_ts = chrono::Utc::now().timestamp();
    certs
        .iter()
        .enumerate()
        .map(|(depth, der)| match parse_x509_certificate(der.as_ref()) {
            Ok((_, cert)) => {
                let validity = cert.validity();
                CertInfo {
                    depth,
                    subject: cn_of(cert.subject()).unwrap_or_else(|| cert.subject().to_string()),
                    issuer: cn_of(cert.issuer()).unwrap_or_else(|| cert.issuer().to_string()),
                    serial: cert.raw_serial_as_string(),
                    not_before: fmt_date(validity.not_before.timestamp()),
                    not_after: fmt_date(validity.not_after.timestamp()),
                    is_ca: cert
                        .basic_constraints()
                        .map(|bc| bc.map(|b| b.value.ca).unwrap_or(false))
                        .unwrap_or(false),
                    expired: now_ts > validity.not_after.timestamp(),
                }
            }
            Err(_) => CertInfo {
                depth,
                subject: "(证书解析失败)".to_string(),
                issuer: "?".to_string(),
                serial: "?".to_string(),
                not_before: "?".to_string(),
                not_after: "?".to_string(),
                is_ca: false,
                expired: false,
            },
        })
        .collect()
}

/// 取证书主题/签发者的 CN（OID 2.5.4.3），取不到回退到完整主题。
fn cn_of(name: &X509Name) -> Option<String> {
    name.iter_common_name().find_map(|attr| attr.as_str().ok().map(|s| s.to_string()))
}

fn fmt_date(ts: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "?".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn parse_real_chain_github() {
        let r = handshake("api.github.com", 443, "api.github.com").await;
        assert!(r.ok, "握手应成功: {:?}", r.error);
        assert!(!r.chain.is_empty(), "应有证书链");
        for c in &r.chain {
            println!(
                "#{} subject={} issuer={} ca={} valid={}~{} expired={}",
                c.depth, c.subject, c.issuer, c.is_ca, c.not_before, c.not_after, c.expired
            );
        }
    }

    #[test]
    fn no_certs_yields_empty_chain() {
        assert!(build_chain_info(&[]).is_empty());
    }
}