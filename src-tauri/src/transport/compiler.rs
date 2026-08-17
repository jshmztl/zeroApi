//! RequestCompiler（文档 §12）
//!
//! 职责：把 `Request + Environment + Secret + Settings` 编译为与传输层解耦的
//! `CompiledRequest`。Transport 不再负责业务变量替换。
//!
//! 处理顺序：
//! 1. Environment substitution（{{var}} 占位符）
//! 2. URL parse（含 base_url 前缀）
//! 3. Query merge
//! 4. Auth
//! 5. Header merge
//! 6. Body compile
//!
//! 编译错误返回结构化 `NetworkError`。

use std::collections::HashMap;

use url::Url;

use crate::domain::*;

/// 编译上下文
pub struct CompileContext<'a> {
    /// 当前激活环境（Secret 与 Plain 变量均参与编译）
    pub env: Option<&'a Environment>,
    pub settings: &'a Settings,
}

/// 编译后的请求体
pub enum CompiledBody {
    /// 原始字节体（raw / urlencoded），携带 content-type
    Bytes { content_type: String, data: Vec<u8> },
    /// multipart 文本字段（form-data）
    Multipart(Vec<(String, String)>),
}

/// 编译产物（不依赖 reqwest）
pub struct CompiledRequest {
    pub method: HttpMethod,
    pub url: Url,
    /// 有序 Header 列表（后项覆盖前项，Auth 追加在用户 Header 之后）
    pub headers: Vec<(String, String)>,
    pub body: Option<CompiledBody>,
}

/// 替换文本中的 {{var}} 占位符（未命中保留原样）
pub fn substitute_vars(input: &str, vars: &HashMap<String, String>) -> String {
    let mut out = input.to_string();
    for (k, v) in vars {
        let token = format!("{{{{{}}}}}", k);
        out = out.replace(&token, v);
    }
    out
}

/// 编译请求
pub fn compile(
    request: &Request,
    ctx: &CompileContext,
) -> Result<CompiledRequest, NetworkError> {
    let vars = ctx
        .env
        .map(|e| e.collect_vars())
        .unwrap_or_default();

    // 1. URL：变量替换 + base_url 前缀
    let mut url_str = substitute_vars(&request.url, &vars);
    if url_str.starts_with('/') {
        if let Some(base) = vars.get("baseUrl").or_else(|| vars.get("base_url")) {
            if !base.is_empty() {
                url_str = format!("{}{}", base.trim_end_matches('/'), url_str);
            }
        }
    }
    let mut url = Url::parse(&url_str).map_err(|e| {
        NetworkError::with_detail(
            NetworkErrorKind::InvalidUrl,
            format!("URL 解析失败: {}", e),
            url_str,
        )
    })?;

    // 2. Query merge
    for kv in &request.query {
        if !kv.enabled || kv.name.is_empty() {
            continue;
        }
        let name = substitute_vars(&kv.name, &vars);
        let value = substitute_vars(&kv.value, &vars);
        url.query_pairs_mut().append_pair(&name, &value);
    }

    // 3. 用户 Header（先收集，Auth 后追加并覆盖同名）
    let mut headers: Vec<(String, String)> = Vec::new();
    for h in &request.headers {
        if h.name.is_empty() {
            continue;
        }
        headers.push((
            substitute_vars(&h.name, &vars),
            substitute_vars(&h.value, &vars),
        ));
    }

    // 4. Auth
    match request.effective_auth() {
        AuthConfig::None => {}
        AuthConfig::Bearer { token } => {
            let t = substitute_vars(&token, &vars);
            if !t.is_empty() {
                headers.push(("authorization".to_string(), format!("Bearer {}", t)));
            }
        }
        AuthConfig::Basic { username, password } => {
            let u = substitute_vars(&username, &vars);
            let p = substitute_vars(&password, &vars);
            let creds = format!("{}:{}", u, p);
            let encoded = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                creds.as_bytes(),
            );
            headers.push(("authorization".to_string(), format!("Basic {}", encoded)));
        }
        AuthConfig::ApiKey { key, value, location } => {
            let k = substitute_vars(&key, &vars);
            let v = substitute_vars(&value, &vars);
            if !k.is_empty() {
                if location.eq_ignore_ascii_case("query") {
                    url.query_pairs_mut().append_pair(&k, &v);
                } else {
                    headers.push((k, v));
                }
            }
        }
    }

    // 5. Body compile
    let body = match request.effective_body() {
        RequestBody::None => None,
        RequestBody::FormData { items } => {
            let fields: Vec<(String, String)> = items
                .iter()
                .filter(|kv| kv.enabled && !kv.name.is_empty())
                .map(|kv| {
                    (
                        substitute_vars(&kv.name, &vars),
                        substitute_vars(&kv.value, &vars),
                    )
                })
                .collect();
            Some(CompiledBody::Multipart(fields))
        }
        RequestBody::UrlEncoded { items } => {
            let pairs: Vec<(String, String)> = items
                .iter()
                .filter(|kv| kv.enabled && !kv.name.is_empty())
                .map(|kv| {
                    (
                        substitute_vars(&kv.name, &vars),
                        substitute_vars(&kv.value, &vars),
                    )
                })
                .collect();
            let mut data = String::new();
            for (i, (k, v)) in pairs.iter().enumerate() {
                if i > 0 {
                    data.push('&');
                }
                data.push_str(&url::form_urlencoded::byte_serialize(k.as_bytes()).collect::<String>());
                data.push('=');
                data.push_str(&url::form_urlencoded::byte_serialize(v.as_bytes()).collect::<String>());
            }
            Some(CompiledBody::Bytes {
                content_type: "application/x-www-form-urlencoded".to_string(),
                data: data.into_bytes(),
            })
        }
        RequestBody::Raw { content_type, content } => {
            let content = substitute_vars(&content, &vars);
            let ct = substitute_vars(&content_type, &vars);
            Some(CompiledBody::Bytes {
                content_type: if ct.is_empty() {
                    "text/plain".to_string()
                } else {
                    ct
                },
                data: content.into_bytes(),
            })
        }
    };

    Ok(CompiledRequest {
        method: request.method,
        url,
        headers,
        body,
    })
}
