//! cURL 命令解析器（V2 模型）
//!
//! 支持：
//! - `-X METHOD` / `--request`
//! - `-H "Key: Value"` / `--header`
//! - `-d 'data'` / `--data` / `--data-raw` / `--data-binary`
//! - `--data-urlencode`
//! - `-u user:pass` / `--user` (Basic Auth)
//! - `-b "cookie"` / `--cookie`
//! - `-F "key=val"` / `--form` (multipart)
//! - `--url` / 末尾位置参数
//! - `-G` / `--get` (将 -d 数据并入 query)
//! - 引号(单/双)、反斜杠转义
//!
//! 不支持：配置文件(-K)、复杂 globbing、变量展开

use crate::domain::*;
use crate::AppError;
use crate::AppResult;

pub fn parse(input: &str) -> AppResult<Request> {
    let trimmed = input.trim();
    let lower = trimmed.to_lowercase();
    if !(lower.starts_with("curl ") || lower.starts_with("curl\t") || lower == "curl") {
        return Err(AppError::Curl("命令必须以 curl 开头".into()));
    }

    let mut tokens = tokenize(trimmed)?;
    if tokens.is_empty() {
        return Err(AppError::Curl("空命令".into()));
    }
    // 去掉开头的 "curl"
    if tokens[0].eq_ignore_ascii_case("curl") {
        tokens.remove(0);
    }

    let mut method: Option<String> = None;
    let mut url: Option<String> = None;
    let mut headers: Vec<HeaderEntry> = Vec::new();
    let mut data_parts: Vec<String> = Vec::new();
    let mut url_encoded_parts: Vec<KeyValue> = Vec::new();
    let mut form_parts: Vec<KeyValue> = Vec::new();
    let mut use_get = false;
    let mut basic_auth: Option<(String, String)> = None;
    let mut cookies: Vec<String> = Vec::new();
    let mut body_type: &str = "none";

    let mut i = 0;
    while i < tokens.len() {
        let t = &tokens[i];
        match t.as_str() {
            // ----- Method -----
            "-X" | "--request" => {
                i += 1;
                if i < tokens.len() {
                    method = Some(tokens[i].to_uppercase());
                }
            }
            // ----- Header -----
            "-H" | "--header" => {
                i += 1;
                if i < tokens.len() {
                    if let Some((k, v)) = split_header(&tokens[i]) {
                        headers.push(HeaderEntry::new(k, v));
                    }
                }
            }
            // ----- Body: raw -----
            "-d" | "--data" | "--data-raw" | "--data-ascii" | "--data-binary" => {
                i += 1;
                if i < tokens.len() {
                    data_parts.push(tokens[i].clone());
                    if body_type == "none" {
                        body_type = "raw";
                    }
                }
            }
            "--data-urlencode" => {
                i += 1;
                if i < tokens.len() {
                    // 支持 "key=value" 与 "key" (value 来自 stdin)
                    if let Some(eq) = tokens[i].find('=') {
                        let k = tokens[i][..eq].to_string();
                        let v = tokens[i][eq + 1..].to_string();
                        url_encoded_parts.push(KeyValue::new(k, v));
                    } else {
                        url_encoded_parts.push(KeyValue::new(tokens[i].clone(), ""));
                    }
                    if body_type == "none" {
                        body_type = "urlencoded";
                    }
                }
            }
            // ----- Body: multipart -----
            "-F" | "--form" | "--form-string" => {
                i += 1;
                if i < tokens.len() {
                    if let Some((k, v)) = split_first_eq(&tokens[i]) {
                        form_parts.push(KeyValue::new(k, v));
                    }
                    body_type = "formdata";
                }
            }
            // ----- Auth -----
            "-u" | "--user" => {
                i += 1;
                if i < tokens.len() {
                    if let Some((u, p)) = tokens[i].split_once(':') {
                        basic_auth = Some((u.to_string(), p.to_string()));
                    } else {
                        basic_auth = Some((tokens[i].clone(), String::new()));
                    }
                }
            }
            // ----- Cookie -----
            "-b" | "--cookie" => {
                i += 1;
                if i < tokens.len() {
                    cookies.push(tokens[i].clone());
                }
            }
            // ----- Get -----
            "-G" | "--get" => {
                use_get = true;
            }
            // ----- URL -----
            "--url" => {
                i += 1;
                if i < tokens.len() {
                    url = Some(tokens[i].clone());
                }
            }
            // ----- 未知 flag（跳过，如 -L -k -i --compressed）-----
            _ => {
                // 位置参数 -> URL(取第一个遇到的非 flag)
                if url.is_none() && !t.starts_with('-') {
                    url = Some(t.clone());
                }
            }
        }
        i += 1;
    }

    // URL 不能空
    let url = url.ok_or_else(|| AppError::Curl("未找到 URL".into()))?;

    // 把 cookies 合并到 Cookie 头
    if !cookies.is_empty() {
        headers.push(HeaderEntry::new("Cookie", cookies.join("; ")));
    }

    // Basic auth
    let auth = basic_auth.map(|(u, p)| AuthConfig::Basic {
        username: u,
        password: p,
    });

    // 决定最终 method
    let method_str = method
        .or_else(|| {
            if use_get {
                Some("GET".to_string())
            } else if body_type == "none" {
                Some("GET".to_string())
            } else {
                Some("POST".to_string())
            }
        })
        .unwrap_or_else(|| "GET".to_string());
    let http_method = HttpMethod::from_str(&method_str)
        .ok_or_else(|| AppError::Curl(format!("未知 HTTP 方法: {}", method_str)))?;

    // 处理 body 类型
    let body = match body_type {
        "raw" => {
            let content = data_parts.join("&");
            // 智能判断 content type
            let ct = headers
                .iter()
                .find(|h| h.name.eq_ignore_ascii_case("Content-Type"))
                .map(|h| h.value.clone());
            let default_ct = if content.trim_start().starts_with('{')
                || content.trim_start().starts_with('[')
            {
                "application/json"
            } else {
                "application/x-www-form-urlencoded"
            };
            Some(RequestBody::Raw {
                content_type: ct.unwrap_or_else(|| default_ct.to_string()),
                content,
            })
        }
        "urlencoded" => {
            // 合并 -d + --data-urlencode
            for d in &data_parts {
                for kv in parse_kv_string(d) {
                    url_encoded_parts.push(kv);
                }
            }
            Some(RequestBody::UrlEncoded {
                items: url_encoded_parts,
            })
        }
        "formdata" => Some(RequestBody::FormData {
            items: form_parts,
        }),
        _ => None,
    };

    // 如果 -G 且有 -d,把它们转成 query 参数
    let mut extra_params: Vec<KeyValue> = Vec::new();
    if use_get && !data_parts.is_empty() {
        for d in &data_parts {
            extra_params.extend(parse_kv_string(d));
        }
    }

    // 移除 GET 请求里残留的 Content-Length/Content-Type 头
    if method_str == "GET" {
        headers.retain(|h| {
            !(h.name.eq_ignore_ascii_case("Content-Length")
                || h.name.eq_ignore_ascii_case("Content-Type"))
        });
    }

    let now = chrono::Utc::now().timestamp_millis();
    Ok(Request {
        id: String::new(),
        collection_id: String::new(),
        folder_id: None,
        name: String::new(),
        method: http_method,
        url,
        headers,
        query: extra_params,
        body,
        auth,
        sort_order: 0,
        created_at: now,
        updated_at: now,
    })
}

/// 把 Request 转成 cURL 命令（文档 §24：双向转换）
pub fn to_curl(request: &Request) -> String {
    let mut parts: Vec<String> = vec!["curl".to_string()];

    // 方法
    parts.push(format!("-X {}", request.method.as_str()));

    // URL（合并 query 参数与 apiKey in query）
    let mut url = request.url.clone();
    let mut query_pairs: Vec<(String, String)> = request
        .query
        .iter()
        .filter(|q| q.enabled && !q.name.is_empty())
        .map(|q| (q.name.clone(), q.value.clone()))
        .collect();
    if let AuthConfig::ApiKey { key, value, location } = request.effective_auth() {
        if location.eq_ignore_ascii_case("query") && !key.is_empty() {
            query_pairs.push((key, value));
        }
    }
    if !query_pairs.is_empty() {
        let sep = if url.contains('?') { "&" } else { "?" };
        let qs: Vec<String> = query_pairs
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect();
        url = format!("{}{}{}", url, sep, qs.join("&"));
    }
    parts.push(shell_escape(&url));

    // Headers
    for h in &request.headers {
        parts.push(format!("-H {}", shell_escape(&format!("{}: {}", h.name, h.value))));
    }

    // Auth
    match request.effective_auth() {
        AuthConfig::None => {}
        AuthConfig::Bearer { token } if !token.is_empty() => {
            parts.push(format!(
                "-H {}",
                shell_escape(&format!("Authorization: Bearer {}", token))
            ));
        }
        AuthConfig::Basic { username, password } => {
            parts.push(format!("-u {}", shell_escape(&format!("{}:{}", username, password))));
        }
        AuthConfig::ApiKey { key, value, location } if location.eq_ignore_ascii_case("header") && !key.is_empty() => {
            parts.push(format!("-H {}", shell_escape(&format!("{}: {}", key, value))));
        }
        _ => {}
    }

    // Body
    match request.effective_body() {
        RequestBody::None => {}
        RequestBody::Raw { content, .. } if !content.is_empty() => {
            parts.push(format!("--data-raw {}", shell_escape(&content)));
        }
        RequestBody::UrlEncoded { items } => {
            let pairs: Vec<String> = items
                .iter()
                .filter(|kv| kv.enabled && !kv.name.is_empty())
                .map(|kv| format!("{}={}", urlencoding::encode(&kv.name), urlencoding::encode(&kv.value)))
                .collect();
            if !pairs.is_empty() {
                parts.push(format!("--data {}", shell_escape(&pairs.join("&"))));
            }
        }
        RequestBody::FormData { items } => {
            for kv in items.iter().filter(|kv| kv.enabled && !kv.name.is_empty()) {
                parts.push(format!("-F {}", shell_escape(&format!("{}={}", kv.name, kv.value))));
            }
        }
        _ => {}
    }

    parts.join(" ")
}

/// Shell 单引号包裹转义：将值用单引号包裹，内部单引号转为 `'\''`
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// 把 token 拆成 shell token,处理引号与转义
fn tokenize(input: &str) -> AppResult<Vec<String>> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if escape {
            cur.push(c);
            escape = false;
            i += 1;
            continue;
        }
        match c {
            '\\' if in_single => {
                // 单引号内反斜杠不转义,保留原字符
                cur.push(c);
            }
            '\\' => {
                escape = true;
            }
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            c if c.is_whitespace() && !in_single && !in_double => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
        i += 1;
    }
    if in_single || in_double {
        return Err(AppError::Curl("引号未闭合".into()));
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    Ok(out)
}

fn split_header(s: &str) -> Option<(String, String)> {
    let idx = s.find(':')?;
    let key = s[..idx].trim().to_string();
    let value = s[idx + 1..].trim().to_string();
    Some((key, value))
}

fn split_first_eq(s: &str) -> Option<(String, String)> {
    let idx = s.find('=')?;
    Some((s[..idx].to_string(), s[idx + 1..].to_string()))
}

fn parse_kv_string(s: &str) -> Vec<KeyValue> {
    s.split('&')
        .filter_map(|p| {
            if p.is_empty() {
                return None;
            }
            let (k, v) = match p.find('=') {
                Some(idx) => (p[..idx].to_string(), p[idx + 1..].to_string()),
                None => (p.to_string(), String::new()),
            };
            let v = urlencoding_decode(&v);
            Some(KeyValue::new(urlencoding_decode(&k), v))
        })
        .collect()
}

fn urlencoding_decode(s: &str) -> String {
    urlencoding::decode(s)
        .map(|c| c.into_owned())
        .unwrap_or_else(|_| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_get() {
        let r = parse("curl https://api.example.com/users").unwrap();
        assert_eq!(r.method.as_str(), "GET");
        assert_eq!(r.url, "https://api.example.com/users");
    }

    #[test]
    fn test_post_with_header_and_body() {
        let r = parse(r#"curl -X POST -H "Content-Type: application/json" -d '{"name":"Alice"}' https://api.example.com/users"#).unwrap();
        assert_eq!(r.method.as_str(), "POST");
        assert_eq!(r.url, "https://api.example.com/users");
        assert!(r.headers.iter().any(|h| h.name == "Content-Type"));
    }

    #[test]
    fn test_basic_auth() {
        let r = parse("curl -u admin:secret https://api.example.com").unwrap();
        match r.auth {
            Some(AuthConfig::Basic { username, password }) => {
                assert_eq!(username, "admin");
                assert_eq!(password, "secret");
            }
            _ => panic!("expected basic auth"),
        }
    }

    #[test]
    fn test_data_with_get() {
        let r = parse("curl -G -d \"a=1&b=2\" https://api.example.com").unwrap();
        assert_eq!(r.method.as_str(), "GET");
        assert_eq!(r.query.len(), 2);
    }

    #[test]
    fn test_cookie_header() {
        let r = parse("curl -b \"sid=abc\" https://api.example.com").unwrap();
        assert!(r.headers.iter().any(|h| h.name == "Cookie" && h.value == "sid=abc"));
    }

    #[test]
    fn test_to_curl_roundtrip() {
        let r = parse(r#"curl -X POST -H "Content-Type: application/json" -d '{"name":"Alice"}' https://api.example.com/users"#).unwrap();
        let c = to_curl(&r);
        assert!(c.starts_with("curl"));
        assert!(c.contains("-X POST"));
        assert!(c.contains("https://api.example.com/users"));
        // 再解析回去，关键信息保留
        let r2 = parse(&c).unwrap();
        assert_eq!(r2.method, r.method);
        assert_eq!(r2.url, r.url);
        assert!(r2.headers.iter().any(|h| h.name == "Content-Type"));
    }

    #[test]
    fn test_to_curl_query_and_auth() {
        let r = parse("curl -G -d \"a=1\" -u admin:secret https://api.example.com").unwrap();
        let c = to_curl(&r);
        assert!(c.contains("-u 'admin:secret'"));
        assert!(c.contains("a=1"));
    }

    #[test]
    fn test_shell_escape_single_quote() {
        assert_eq!(shell_escape("it's"), "'it'\\''s'");
    }

    #[test]
    fn test_to_curl_url_encoding() {
        let mut req = Request {
            id: String::new(),
            collection_id: String::new(),
            folder_id: None,
            name: "test".into(),
            method: HttpMethod::Get,
            url: "https://api.example.com".into(),
            headers: vec![],
            query: vec![KeyValue::new("key", "value with spaces & stuff=")],
            body: None,
            auth: None,
            sort_order: 0,
            created_at: 0,
            updated_at: 0,
        };
        let c = to_curl(&req);
        assert!(c.contains("key=value+with+spaces+%26+stuff%3D"), "query 参数应被 URL 编码");
    }

    #[test]
    fn test_shell_escape_command_injection_safety() {
        let malicious = "$(rm -rf /)";
        let escaped = shell_escape(malicious);
        // 单引号内 $、`、\ 均不被 shell 解释，只有单引号本身需要转义
        assert_eq!(escaped, "'$(rm -rf /)'");
    }
}
