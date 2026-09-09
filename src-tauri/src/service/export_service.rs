//! Export 服务：生成可进入 Git 的 JSON 备份
//!
//! 原则（文档 §22）：不导出 Secret 明文值、请求历史、Cookie、本地设置。

use crate::domain::{AuthConfig, ExportPayload, Favorite, HeaderEntry};
use crate::repository::Repos;
use crate::AppResult;

#[derive(Clone)]
pub struct ExportService {
    repos: Repos,
}

impl ExportService {
    pub fn new(repos: &Repos) -> Self {
        Self {
            repos: repos.clone(),
        }
    }

    pub fn export(&self) -> AppResult<String> {
        let mut payload = ExportPayload::empty();
        payload.projects = self.repos.projects.list()?;
        payload.collections = self.repos.collections.list(None)?;
        payload.folders = self.repos.folders.list(None)?;
        payload.requests = self.repos.requests.list(None, None)?;
        payload.environments = sanitize_environments(self.repos.environments.list()?);
        payload.favorites = sanitize_favorites(self.repos.requests.list_favorites()?);
        Ok(serde_json::to_string_pretty(&payload)?)
    }
}

/// 导出时剔除 Secret 变量的明文值（只保留名称与类型）
fn sanitize_environments(
    envs: Vec<crate::domain::Environment>,
) -> Vec<crate::domain::Environment> {
    envs.into_iter()
        .map(|mut e| {
            for v in e.vars.iter_mut() {
                if v.kind == crate::domain::VariableKind::Secret {
                    v.value.clear();
                }
            }
            e
        })
        .collect()
}

/// 敏感请求头名（大小写不敏感）
const SENSITIVE_HEADER_NAMES: &[&str] = &[
    "authorization",
    "cookie",
    "set-cookie",
    "x-api-key",
    "x-auth-token",
    "x-access-token",
    "api-key",
    "token",
    "x-csrf-token",
    "x-xsrf-token",
    "x-requested-with",
];

/// 导出时对收藏请求中的敏感凭证进行脱敏
fn sanitize_favorites(favs: Vec<Favorite>) -> Vec<Favorite> {
    favs.into_iter()
        .map(|mut fav| {
            sanitize_request(&mut fav.request);
            fav
        })
        .collect()
}

/// 对单个请求进行脱敏处理
fn sanitize_request(req: &mut crate::domain::Request) {
    // 1. 脱敏敏感请求头
    for h in req.headers.iter_mut() {
        if is_sensitive_header(&h.name) {
            h.value = "***".to_string();
        }
    }

    // 2. 脱敏鉴权信息
    req.auth = req.auth.as_ref().map(|auth| match auth {
        AuthConfig::Bearer { .. } => AuthConfig::Bearer {
            token: "***".to_string(),
        },
        AuthConfig::Basic { username, .. } => AuthConfig::Basic {
            username: username.clone(),
            password: "***".to_string(),
        },
        AuthConfig::ApiKey { key, location, .. } => AuthConfig::ApiKey {
            key: key.clone(),
            value: "***".to_string(),
            location: location.clone(),
        },
        AuthConfig::None => AuthConfig::None,
    });

    // 3. 脱敏 URL query 中的 API Key（如果 auth 是 ApiKey in query）
    if let Some(AuthConfig::ApiKey { key, .. }) = &req.auth {
        let key_lower = key.to_lowercase();
        for q in req.query.iter_mut() {
            if q.enabled && q.name.to_lowercase() == key_lower {
                q.value = "***".to_string();
            }
        }
    }
}

fn is_sensitive_header(name: &str) -> bool {
    let lower = name.to_lowercase();
    SENSITIVE_HEADER_NAMES.iter().any(|&s| lower == s)
        || lower.contains("auth")
        || lower.contains("token")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("credential")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AuthConfig, HeaderEntry, HttpMethod, Request};

    #[test]
    fn test_sanitize_request_headers() {
        let mut req = Request {
            id: String::new(),
            collection_id: String::new(),
            folder_id: None,
            name: "test".into(),
            method: HttpMethod::Get,
            url: "https://api.example.com".into(),
            headers: vec![
                HeaderEntry::new("Authorization", "Bearer secret-token"),
                HeaderEntry::new("Content-Type", "application/json"),
                HeaderEntry::new("Cookie", "session=abc123"),
                HeaderEntry::new("X-Api-Key", "super-secret"),
                HeaderEntry::new("X-Custom-Auth", "custom-value"),
            ],
            query: vec![],
            body: None,
            auth: None,
            sort_order: 0,
            created_at: 0,
            updated_at: 0,
        };
        sanitize_request(&mut req);

        assert_eq!(req.headers[0].value, "***"); // Authorization
        assert_eq!(req.headers[1].value, "application/json"); // Content-Type 不脱敏
        assert_eq!(req.headers[2].value, "***"); // Cookie
        assert_eq!(req.headers[3].value, "***"); // X-Api-Key
        assert_eq!(req.headers[4].value, "***"); // X-Custom-Auth
    }

    #[test]
    fn test_sanitize_request_auth() {
        let mut req = Request {
            id: String::new(),
            collection_id: String::new(),
            folder_id: None,
            name: "test".into(),
            method: HttpMethod::Get,
            url: "https://api.example.com".into(),
            headers: vec![],
            query: vec![],
            body: None,
            auth: Some(AuthConfig::Basic {
                username: "admin".into(),
                password: "secret123".into(),
            }),
            sort_order: 0,
            created_at: 0,
            updated_at: 0,
        };
        sanitize_request(&mut req);

        match req.auth {
            Some(AuthConfig::Basic { username, password }) => {
                assert_eq!(username, "admin");
                assert_eq!(password, "***");
            }
            _ => panic!("expected Basic auth"),
        }
    }
}
