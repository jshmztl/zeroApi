//! HttpTransport（文档 §13）
//!
//! 把 `CompiledRequest` 发送并返回 `ResponseSnapshot`。
//! - 错误分类为结构化 `NetworkErrorKind`
//! - 大响应保护（max_preview_size，超限保存为文件）
//! - 重复 Header 保留（Vec<HeaderEntry>）
//! - Cookie 通过外部注入的 `Jar` 隔离（见 session.rs）

use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use crate::domain::*;
use crate::transport::compiler::{CompiledBody, CompiledRequest};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

pub struct HttpTransport {
    allow_invalid_certs: bool,
    proxy: Option<reqwest::Proxy>,
    user_agent: &'static str,
    max_preview_size: u64,
}

impl HttpTransport {
    pub fn new(settings: &Settings) -> Result<Self, crate::AppError> {
        let proxy = if settings.proxy_url.is_empty() {
            None
        } else {
            Some(reqwest::Proxy::all(&settings.proxy_url)?)
        };
        Ok(Self {
            allow_invalid_certs: !settings.verify_ssl,
            proxy,
            user_agent: "ZeroApi/2.0",
            max_preview_size: settings.max_preview_size.max(1),
        })
    }

    /// 构建带可选 Cookie Jar 的 client（Session 缓存用）
    pub fn build_client(
        &self,
        jar: Option<Arc<reqwest::cookie::Jar>>,
    ) -> Result<reqwest::Client, reqwest::Error> {
        let mut b = reqwest::Client::builder()
            .user_agent(self.user_agent)
            .danger_accept_invalid_certs(self.allow_invalid_certs)
            .redirect(reqwest::redirect::Policy::none());
        if let Some(p) = &self.proxy {
            b = b.proxy(p.clone());
        }
        if let Some(j) = jar {
            b = b.cookie_provider(j);
        }
        b.build()
    }

    /// 执行请求
    pub async fn execute(
        &self,
        client: &reqwest::Client,
        compiled: &CompiledRequest,
        timeout_ms: u64,
        base_dir: &Path,
    ) -> Result<ResponseSnapshot, NetworkError> {
        let start = Instant::now();

        // Phase 4：分段探测（DNS / TCP / TLS），供 Timing 展示
        // 探测值近似（reqwest 内部会再次建连），对诊断与内网场景足够
        let (dns_ms, tcp_ms, tls_ms) = probe_phases(&compiled.url).await;

        let mut req = client
            .request(method_to_reqwest(compiled.method), compiled.url.clone())
            .timeout(std::time::Duration::from_millis(timeout_ms));

        // Headers（后项覆盖前项）
        let mut header_map = HeaderMap::new();
        for (k, v) in &compiled.headers {
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(v),
            ) {
                header_map.insert(name, value);
            }
        }
        req = req.headers(header_map);

        // Body
        match &compiled.body {
            None => {}
            Some(CompiledBody::Bytes { content_type, data }) => {
                req = req.header(reqwest::header::CONTENT_TYPE, content_type);
                req = req.body(data.clone());
            }
            Some(CompiledBody::Multipart(fields)) => {
                let mut form = reqwest::multipart::Form::new();
                for (k, v) in fields {
                    form = form.text(k.clone(), v.clone());
                }
                req = req.multipart(form);
            }
        }

        // 发送（t0 = 请求发出，t1 = 响应头到达）
        let t0 = Instant::now();
        let resp = req.send().await.map_err(|e| classify_error(&e))?;
        let t1 = Instant::now();
        let elapsed = start.elapsed().as_millis() as u64;

        let status = resp.status().as_u16();
        let status_text = resp.status().canonical_reason().unwrap_or("").to_string();

        // Headers（保留重复）
        let mut headers: Vec<HeaderEntry> = Vec::new();
        let mut content_type: Option<String> = None;
        for (k, v) in resp.headers().iter() {
            let key = k.to_string();
            let val = v.to_str().unwrap_or("").to_string();
            if k == reqwest::header::CONTENT_TYPE {
                content_type = Some(val.clone());
            }
            headers.push(HeaderEntry::new(key, val));
        }

        // Body（大响应保护）
        let bytes = resp.bytes().await.map_err(|e| classify_error(&e))?;
        let size_bytes = bytes.len() as u64;
        let t2 = Instant::now();

        let body = if size_bytes > self.max_preview_size {
            // 超限：保存到文件，UI 只展示元信息
            let path = save_response_file(base_dir, &bytes).map_err(|e| {
                NetworkError::with_detail(
                    NetworkErrorKind::Unknown,
                    "响应体保存失败".to_string(),
                    e.to_string(),
                )
            })?;
            ResponseBody::Binary {
                path,
                size: size_bytes,
            }
        } else {
            ResponseBody::Text { text: String::from_utf8_lossy(&bytes).into_owned() }
        };

        Ok(ResponseSnapshot {
            status,
            status_text,
            headers,
            body,
            size_bytes,
            content_type,
            timing: Timing {
                dns_ms,
                tcp_ms,
                tls_ms,
                // 请求发出到响应头到达
                request_ms: Some(t1.duration_since(t0).as_millis() as u64),
                // 响应头到响应体完成
                response_ms: Some(t2.duration_since(t1).as_millis() as u64),
                total_ms: elapsed,
            },
        })
    }
}

/// 分段探测（DNS / TCP / TLS），返回 (dns_ms, tcp_ms, tls_ms)
pub(crate) async fn probe_phases(url: &url::Url) -> (Option<u64>, Option<u64>, Option<u64>) {
    let host = url.host_str().unwrap_or("").to_string();
    if host.is_empty() {
        return (None, None, None);
    }
    let scheme = url.scheme().to_string();
    let port = url.port().unwrap_or_else(|| {
        if scheme == "https" || scheme == "wss" {
            443
        } else {
            80
        }
    });

    // DNS
    let dns = crate::network::dns::resolve(&host).await;
    let dns_ms = if dns.ok { Some(dns.ms) } else { None };

    // TCP
    let tcp = crate::network::tcp::connect(&host, port).await;
    let tcp_ms = if tcp.ok { Some(tcp.ms) } else { None };

    // TLS（仅 https / wss 且 TCP 成功）
    let tls_ms = if (scheme == "https" || scheme == "wss") && tcp.ok {
        let tls = crate::network::tls::handshake(&host, port, &host).await;
        if tls.ok {
            Some(tls.ms)
        } else {
            None
        }
    } else {
        None
    };

    (dns_ms, tcp_ms, tls_ms)
}

/// 把响应体保存为文件（目录不存在则创建），返回路径
fn save_response_file(base_dir: &Path, bytes: &[u8]) -> std::io::Result<String> {
    std::fs::create_dir_all(base_dir)?;
    let name = format!(
        "resp-{}-{}.bin",
        chrono::Utc::now().timestamp_millis(),
        uuid::Uuid::new_v4().simple()
    );
    let path = base_dir.join(name);
    std::fs::write(&path, bytes)?;
    Ok(path.to_string_lossy().into_owned())
}

fn method_to_reqwest(m: HttpMethod) -> reqwest::Method {
    match m {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Head => reqwest::Method::HEAD,
        HttpMethod::Options => reqwest::Method::OPTIONS,
    }
}

/// 把 reqwest 错误分类为结构化 NetworkError
fn classify_error(e: &reqwest::Error) -> NetworkError {
    if e.is_timeout() {
        return NetworkError::new(NetworkErrorKind::Timeout, e.to_string());
    }
    if e.is_connect() {
        let chain = error_chain_text(e).to_lowercase();
        if chain.contains("dns")
            || chain.contains("resolve")
            || chain.contains("lookup")
            || chain.contains("nodename")
            || chain.contains("name or service not known")
        {
            return NetworkError::new(NetworkErrorKind::Dns, e.to_string());
        }
        return NetworkError::new(NetworkErrorKind::Connection, e.to_string());
    }
    if e.is_builder() {
        return NetworkError::new(NetworkErrorKind::InvalidUrl, e.to_string());
    }
    let chain = error_chain_text(e).to_lowercase();
    if chain.contains("certificate")
        || chain.contains("tls")
        || chain.contains("handshake")
        || chain.contains("ssl")
    {
        return NetworkError::new(NetworkErrorKind::Tls, e.to_string());
    }
    if chain.contains("proxy") {
        return NetworkError::new(NetworkErrorKind::Proxy, e.to_string());
    }
    if e.is_redirect() || e.is_decode() || e.is_body() || e.is_request() {
        return NetworkError::new(NetworkErrorKind::Protocol, e.to_string());
    }
    NetworkError::new(NetworkErrorKind::Unknown, e.to_string())
}

fn error_chain_text(e: &reqwest::Error) -> String {
    let mut parts = vec![e.to_string()];
    let mut source = e.source();
    while let Some(s) = source {
        parts.push(s.to_string());
        source = s.source();
    }
    parts.join(" | ")
}
