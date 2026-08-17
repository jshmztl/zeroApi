//! Request 领域模型（V2）
//!
//! V2 重构要点：
//! - Request 不再包含 `last_response` / `last_status` / `status`
//! - Request 与 Response 生命周期完全解耦
//! - 执行记录通过 `RequestExecution` 独立保存

use serde::{Deserialize, Serialize};

use crate::domain::{HeaderEntry, HttpMethod, KeyValue};

/// 请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequestBody {
    None,
    FormData { items: Vec<KeyValue> },
    UrlEncoded { items: Vec<KeyValue> },
    Raw { content_type: String, content: String },
}

impl Default for RequestBody {
    fn default() -> Self {
        RequestBody::None
    }
}

impl RequestBody {
    pub fn is_empty(&self) -> bool {
        matches!(self, RequestBody::None)
    }
}

/// 鉴权配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthConfig {
    None,
    Bearer { token: String },
    Basic { username: String, password: String },
    ApiKey {
        key: String,
        value: String,
        location: String,
    },
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig::None
    }
}

/// 请求（V2）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub collection_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<HeaderEntry>,
    #[serde(default)]
    pub query: Vec<KeyValue>,
    #[serde(default)]
    pub body: Option<RequestBody>,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub sort_order: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            id: String::new(),
            collection_id: String::new(),
            folder_id: None,
            name: String::new(),
            method: HttpMethod::Get,
            url: String::new(),
            headers: Vec::new(),
            query: Vec::new(),
            body: None,
            auth: None,
            sort_order: 0,
            created_at: 0,
            updated_at: 0,
        }
    }
}

impl Request {
    /// 请求体展开：None 时当作空体
    pub fn effective_body(&self) -> RequestBody {
        self.body.clone().unwrap_or(RequestBody::None)
    }

    /// 鉴权展开：None 时当作无鉴权
    pub fn effective_auth(&self) -> AuthConfig {
        self.auth.clone().unwrap_or(AuthConfig::None)
    }
}

/// 收藏（收藏的请求为完整 Request 快照）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    pub id: String,
    pub request: Request,
    pub created_at: i64,
}
