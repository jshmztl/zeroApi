//! 数据模型定义
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP 方法
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(HttpMethod::Get),
            "POST" => Some(HttpMethod::Post),
            "PUT" => Some(HttpMethod::Put),
            "PATCH" => Some(HttpMethod::Patch),
            "DELETE" => Some(HttpMethod::Delete),
            "HEAD" => Some(HttpMethod::Head),
            "OPTIONS" => Some(HttpMethod::Options),
            _ => None,
        }
    }
}

/// 键值对
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub enabled: bool,
}

/// 请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Body {
    None,
    FormData { items: Vec<KeyValue> },
    UrlEncoded { items: Vec<KeyValue> },
    Raw { content_type: String, content: String },
}

impl Default for Body {
    fn default() -> Self {
        Body::None
    }
}

/// 鉴权
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Auth {
    None,
    Bearer { token: String },
    Basic { username: String, password: String },
    ApiKey { key: String, value: String, location: String },
}

impl Default for Auth {
    fn default() -> Self {
        Auth::None
    }
}

/// 请求结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Request {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub params: Vec<KeyValue>,
    #[serde(default)]
    pub headers: Vec<KeyValue>,
    #[serde(default)]
    pub body: Body,
    #[serde(default)]
    pub auth: Auth,
    #[serde(default)]
    pub collection_id: Option<String>,
}

/// 响应快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseSnapshot {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub time_ms: u64,
    pub size_bytes: u64,
    pub content_type: String,
}

/// 历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: String,
    pub request: Request,
    pub response: ResponseSnapshot,
    pub created_at: i64,
}

/// 集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub description: String,
    pub request_ids: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 收藏
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    pub id: String,
    pub request: Request,
    pub created_at: i64,
}

/// 环境变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub vars: Vec<KeyValue>,
    #[serde(default)]
    pub active: bool,
}

/// 设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub proxy_url: String,
    #[serde(default)]
    pub theme: String, // light | dark | system
    #[serde(default = "default_true")]
    pub auto_save_history: bool,
    #[serde(default = "default_history_limit")]
    pub history_limit: u32,
    #[serde(default = "default_true")]
    pub verify_ssl: bool,
    #[serde(default)]
    pub follow_redirects: bool,
}

fn default_timeout() -> u64 { 30000 }
fn default_true() -> bool { true }
fn default_history_limit() -> u32 { 100 }

impl Default for Settings {
    fn default() -> Self {
        Self {
            timeout_ms: 30000,
            proxy_url: String::new(),
            theme: "light".to_string(),
            auto_save_history: true,
            history_limit: 100,
            verify_ssl: true,
            follow_redirects: true,
        }
    }
}

/// 导入/导出容器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPayload {
    pub version: u32,
    pub requests: Vec<Request>,
    pub collections: Vec<Collection>,
    pub environments: Vec<Environment>,
    pub favorites: Vec<Favorite>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPayload {
    pub requests: Vec<Request>,
    pub collections: Vec<Collection>,
    pub environments: Vec<Environment>,
    pub favorites: Vec<Favorite>,
}
