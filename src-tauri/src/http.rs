//! HTTP 客户端
//!
//! 把 Request 转为 reqwest 请求并执行,记录耗时 / 状态 / 大小 / Headers / Body
use std::collections::HashMap;
use std::time::Instant;

use crate::models::*;
use crate::AppResult;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, COOKIE, SET_COOKIE};

/// 替换文本中的 {{var}} 占位符
pub fn substitute_vars(input: &str, vars: &HashMap<String, String>) -> String {
    let mut out = input.to_string();
    for (k, v) in vars {
        let token = format!("{{{{{}}}}}", k);
        out = out.replace(&token, v);
    }
    out
}

/// 从激活的环境里收集变量
pub fn collect_env_vars(env: Option<&Environment>) -> HashMap<String, String> {
    let mut m = HashMap::new();
    if let Some(e) = env {
        for kv in &e.vars {
            if kv.enabled {
                m.insert(kv.key.clone(), kv.value.clone());
            }
        }
        // 如果环境配置了 base_url，也作为变量注入
        if !e.base_url.is_empty() {
            m.insert("baseUrl".to_string(), e.base_url.clone());
        }
    }
    m
}

/// 构造 reqwest 请求(并应用环境变量、超时、SSL 等)
pub fn build_request(
    client: &reqwest::Client,
    req: &Request,
    env_vars: &HashMap<String, String>,
    settings: &Settings,
) -> AppResult<reqwest::RequestBuilder> {
    // 1. URL: 合并 query 参数 & 替换变量
    let mut url = substitute_vars(&req.url, env_vars);
    // 如果 URL 以 / 开头且有 baseUrl 变量，自动拼接接口前缀
    if url.starts_with('/') {
        if let Some(base) = env_vars.get("baseUrl").or_else(|| env_vars.get("base_url")) {
            if !base.is_empty() {
                url = format!("{}{}", base.trim_end_matches('/'), url);
            }
        }
    }
    let enabled_params: Vec<&KeyValue> = req.params.iter().filter(|p| p.enabled && !p.key.is_empty()).collect();
    if !enabled_params.is_empty() {
        let mut u = url::Url::parse(&url)?;
        {
            let mut q = u.query_pairs_mut();
            for p in &enabled_params {
                q.append_pair(&substitute_vars(&p.key, env_vars), &substitute_vars(&p.value, env_vars));
            }
        }
        url = u.to_string();
    }

    // 2. Method
    let method = HttpMethod::from_str(&req.method)
        .ok_or_else(|| crate::AppError::Other(format!("未知 HTTP 方法: {}", req.method)))?;

    let mut builder = client
        .request(method_to_reqwest(&method), &url)
        .timeout(std::time::Duration::from_millis(settings.timeout_ms));

    // 3. Headers
    let mut header_map = HeaderMap::new();
    for h in &req.headers {
        if !h.enabled || h.key.is_empty() {
            continue;
        }
        let key = substitute_vars(&h.key, env_vars);
        let val = substitute_vars(&h.value, env_vars);
        if let (Ok(name), Ok(value)) = (HeaderName::from_bytes(key.as_bytes()), HeaderValue::from_str(&val)) {
            header_map.insert(name, value);
        }
    }

    // 4. Auth
    match &req.auth {
        Auth::None => {}
        Auth::Bearer { token } => {
            let token = substitute_vars(token, env_vars);
            if !token.is_empty() {
                header_map.insert(
                    reqwest::header::AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {}", token))?,
                );
            }
        }
        Auth::Basic { username, password } => {
            let u = substitute_vars(username, env_vars);
            let p = substitute_vars(password, env_vars);
            let creds = format!("{}:{}", u, p);
            let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, creds.as_bytes());
            header_map.insert(
                reqwest::header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Basic {}", encoded))?,
            );
        }
        Auth::ApiKey { key, value, location } => {
            let k = substitute_vars(key, env_vars);
            let v = substitute_vars(value, env_vars);
            if !k.is_empty() {
                match location.as_str() {
                    "query" => {
                        if let Ok(mut u) = url::Url::parse(&url) {
                            u.query_pairs_mut().append_pair(&k, &v);
                            builder = builder.query(&[(&k, &v)]);
                            // 注意:builder 已持有 url,再 query 会覆盖。改用最终 url 重设。
                        }
                    }
                    _ => {
                        if let (Ok(name), Ok(value)) = (HeaderName::from_bytes(k.as_bytes()), HeaderValue::from_str(&v)) {
                            header_map.insert(name, value);
                        }
                    }
                }
            }
        }
    }

    builder = builder.headers(header_map);

    // 5. Body
    match &req.body {
        Body::None => {}
        Body::FormData { items } => {
            let mut form = reqwest::multipart::Form::new();
            for kv in items {
                if !kv.enabled || kv.key.is_empty() {
                    continue;
                }
                let k = substitute_vars(&kv.key, env_vars);
                let v = substitute_vars(&kv.value, env_vars);
                form = form.text(k, v);
            }
            builder = builder.multipart(form);
        }
        Body::UrlEncoded { items } => {
            let mut pairs: Vec<(String, String)> = Vec::new();
            for kv in items {
                if !kv.enabled || kv.key.is_empty() {
                    continue;
                }
                pairs.push((
                    substitute_vars(&kv.key, env_vars),
                    substitute_vars(&kv.value, env_vars),
                ));
            }
            builder = builder.form(&pairs);
        }
        Body::Raw { content_type, content } => {
            let content = substitute_vars(content, env_vars);
            let ct = substitute_vars(content_type, env_vars);
            if !ct.is_empty() {
                builder = builder.header(reqwest::header::CONTENT_TYPE, ct);
            }
            builder = builder.body(content);
        }
    }

    Ok(builder)
}

fn method_to_reqwest(m: &HttpMethod) -> reqwest::Method {
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

/// 执行请求并返回响应快照
pub async fn execute(
    client: &reqwest::Client,
    req: &Request,
    env_vars: &HashMap<String, String>,
    settings: &Settings,
) -> AppResult<ResponseSnapshot> {
    let builder = build_request(client, req, env_vars, settings)?;
    let start = Instant::now();
    let resp = builder.send().await?;
    let elapsed = start.elapsed().as_millis() as u64;

    let status = resp.status().as_u16();
    let status_text = resp.status().canonical_reason().unwrap_or("").to_string();
    let mut headers_map: HashMap<String, String> = HashMap::new();
    let mut cookies: Vec<String> = Vec::new();
    let mut content_type = String::new();

    for (k, v) in resp.headers().iter() {
        let key = k.to_string();
        let val = v.to_str().unwrap_or("").to_string();
        if k == SET_COOKIE {
            cookies.push(val.clone());
        }
        if k == reqwest::header::CONTENT_TYPE {
            content_type = val.clone();
        }
        headers_map.insert(key, val);
    }

    let body = resp.text().await?;
    let size_bytes = body.len() as u64;

    if !cookies.is_empty() {
        headers_map.insert(
            COOKIE.as_str().to_string(),
            cookies.join("; "),
        );
    }

    Ok(ResponseSnapshot {
        status,
        status_text,
        headers: headers_map,
        body,
        time_ms: elapsed,
        size_bytes,
        content_type,
    })
}
