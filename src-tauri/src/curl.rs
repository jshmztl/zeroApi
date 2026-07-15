//! cURL 命令解析器
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
//! - `-k` / `--insecure`
//! - `--compressed`
//! - 引号(单/双)、反斜杠转义
//!
//! 不支持：配置文件(-K)、复杂 globbing、变量展开

use crate::models::*;
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
    let mut headers: Vec<KeyValue> = Vec::new();
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
                        headers.push(KeyValue { key: k, value: v, enabled: true });
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
                        url_encoded_parts.push(KeyValue { key: k, value: v, enabled: true });
                    } else {
                        url_encoded_parts.push(KeyValue { key: tokens[i].clone(), value: "".into(), enabled: true });
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
                        form_parts.push(KeyValue { key: k, value: v, enabled: true });
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
            // ----- 短选项串 -----
            s if s.starts_with('-') && s.len() > 1 && !s.starts_with("--") => {
                // 例如 -L, -k, -I, -i, --silent 等
                // 解析组合短选项:每个字符一个 flag
                for c in s.chars().skip(1) {
                    match c {
                        // 跳过值
                        _ => {}
                    }
                }
            }
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
        headers.push(KeyValue {
            key: "Cookie".to_string(),
            value: cookies.join("; "),
            enabled: true,
        });
    }

    // Basic auth
    let auth = if let Some((u, p)) = basic_auth {
        Auth::Basic { username: u, password: p }
    } else {
        Auth::None
    };

    // 决定最终 method
    let final_method = if let Some(m) = method {
        m
    } else if use_get {
        "GET".to_string()
    } else if body_type == "none" {
        "GET".to_string()
    } else {
        "POST".to_string()
    };

    // 处理 body 类型
    let body = match body_type {
        "raw" => {
            let content = data_parts.join("&");
            // 智能判断 content type
            let ct = headers
                .iter()
                .find(|h| h.key.eq_ignore_ascii_case("Content-Type"))
                .map(|h| h.value.clone());
            let default_ct = if content.trim_start().starts_with('{')
                || content.trim_start().starts_with('[')
            {
                "application/json"
            } else {
                "application/x-www-form-urlencoded"
            };
            Body::Raw {
                content_type: ct.unwrap_or_else(|| default_ct.to_string()),
                content,
            }
        }
        "urlencoded" => {
            // 合并 -d + --data-urlencode
            for d in &data_parts {
                for kv in parse_kv_string(d) {
                    url_encoded_parts.push(kv);
                }
            }
            Body::UrlEncoded { items: url_encoded_parts }
        }
        "formdata" => Body::FormData { items: form_parts },
        _ => {
            // 如果 -G,把 -d 数据并入 query
            if use_get && !data_parts.is_empty() {
                let joined = data_parts.join("&");
                for _kv in parse_kv_string(&joined) {
                    headers.retain(|h| !h.key.eq_ignore_ascii_case("Content-Type"));
                    // 这部分会作为 query 拼到 URL,这里用 headers 暂时存一下
                    // 实际上应该走 params,我们在外层处理
                }
            }
            Body::None
        }
    };

    // 如果 -G 且有 -d,把它们转成 query 参数
    let mut extra_params: Vec<KeyValue> = Vec::new();
    if use_get && !data_parts.is_empty() {
        for d in &data_parts {
            extra_params.extend(parse_kv_string(d));
        }
    }

    // 移除 GET 请求里残留的 Content-Length/Content-Type 头
    if final_method == "GET" {
        headers.retain(|h| {
            !(h.key.eq_ignore_ascii_case("Content-Length")
                || h.key.eq_ignore_ascii_case("Content-Type"))
        });
    }

    Ok(Request {
        id: String::new(),
        name: String::new(),
        method: final_method,
        url,
        params: extra_params,
        headers,
        body,
        auth,
        collection_id: None,
        status: RequestStatus::default(),
    })
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
            Some(KeyValue { key: urlencoding_decode(&k), value: v, enabled: true })
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
        assert_eq!(r.method, "GET");
        assert_eq!(r.url, "https://api.example.com/users");
    }

    #[test]
    fn test_post_with_header_and_body() {
        let r = parse(r#"curl -X POST -H "Content-Type: application/json" -d '{"name":"Alice"}' https://api.example.com/users"#).unwrap();
        assert_eq!(r.method, "POST");
        assert_eq!(r.url, "https://api.example.com/users");
        assert!(r.headers.iter().any(|h| h.key == "Content-Type"));
    }

    #[test]
    fn test_basic_auth() {
        let r = parse("curl -u admin:secret https://api.example.com").unwrap();
        match r.auth {
            Auth::Basic { username, password } => {
                assert_eq!(username, "admin");
                assert_eq!(password, "secret");
            }
            _ => panic!("expected basic auth"),
        }
    }

    #[test]
    fn test_data_with_get() {
        let r = parse("curl -G -d \"a=1&b=2\" https://api.example.com").unwrap();
        assert_eq!(r.method, "GET");
        assert_eq!(r.params.len(), 2);
    }
}
