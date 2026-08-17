//! 结构化网络错误模型（V2）
//!
//! 取代简单依赖 reqwest Error 的方式，UI 可区分：
//! DNS Failed / Connection Refused / Timeout / TLS Error / Proxy Error / HTTP 状态码

use serde::{Deserialize, Serialize};

/// 网络错误分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkErrorKind {
    InvalidUrl,
    Dns,
    Connection,
    Timeout,
    Tls,
    Proxy,
    Protocol,
    Cancelled,
    Unknown,
}

impl NetworkErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            NetworkErrorKind::InvalidUrl => "invalid_url",
            NetworkErrorKind::Dns => "dns",
            NetworkErrorKind::Connection => "connection",
            NetworkErrorKind::Timeout => "timeout",
            NetworkErrorKind::Tls => "tls",
            NetworkErrorKind::Proxy => "proxy",
            NetworkErrorKind::Protocol => "protocol",
            NetworkErrorKind::Cancelled => "cancelled",
            NetworkErrorKind::Unknown => "unknown",
        }
    }

    /// 用户可读的中文描述
    pub fn label(&self) -> &'static str {
        match self {
            NetworkErrorKind::InvalidUrl => "URL 无效",
            NetworkErrorKind::Dns => "DNS 解析失败",
            NetworkErrorKind::Connection => "连接失败",
            NetworkErrorKind::Timeout => "请求超时",
            NetworkErrorKind::Tls => "TLS 证书错误",
            NetworkErrorKind::Proxy => "代理错误",
            NetworkErrorKind::Protocol => "协议错误",
            NetworkErrorKind::Cancelled => "请求已取消",
            NetworkErrorKind::Unknown => "未知错误",
        }
    }
}

/// 结构化网络错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkError {
    pub kind: NetworkErrorKind,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl NetworkError {
    pub fn new(kind: NetworkErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            detail: None,
        }
    }

    pub fn with_detail(
        kind: NetworkErrorKind,
        message: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            detail: Some(detail.into()),
        }
    }
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.kind.label(), self.message)
    }
}

impl std::error::Error for NetworkError {}
