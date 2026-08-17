//! Response 领域模型

use serde::{Deserialize, Serialize};

use crate::domain::{HeaderEntry, Timing};

/// 响应快照
///
/// 与 Request 解耦：一次执行产生一个 ResponseSnapshot，
/// 由 RequestExecution 持有。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseSnapshot {
    pub status: u16,
    /// 状态文本（如 "OK"），由传输层从状态码推导
    #[serde(default)]
    pub status_text: String,
    /// 保留重复 Header（如 Set-Cookie）
    pub headers: Vec<HeaderEntry>,
    pub body: ResponseBody,
    pub size_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    pub timing: Timing,
}

/// 响应体
///
/// - Text: 文本响应（受 max_preview_size 限制）
/// - Binary: 二进制 / 超限响应保存为文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseBody {
    Text(String),
    Binary {
        path: String,
        size: u64,
    },
}

impl ResponseBody {
    pub fn is_binary(&self) -> bool {
        matches!(self, ResponseBody::Binary { .. })
    }
}

/// 文本预览大小上限（默认 5MB，见 Settings.max_preview_size）
pub const DEFAULT_MAX_PREVIEW_SIZE: u64 = 5 * 1024 * 1024;
