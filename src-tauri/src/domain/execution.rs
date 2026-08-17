//! RequestExecution 领域模型
//!
//! 一次请求执行记录，独立于 Request 保存，用于：
//! - 请求历史
//! - 性能统计
//! - Replay
//! - Response Diff
//! - Timeline
//! - 错误诊断

use serde::{Deserialize, Serialize};

use crate::domain::{HttpMethod, NetworkError, ResponseSnapshot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestExecution {
    pub id: String,
    pub request_id: String,
    /// 请求时刻（epoch 毫秒）
    pub started_at: i64,
    pub duration_ms: u64,
    pub status_code: Option<u16>,
    pub success: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<NetworkError>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: Option<ResponseSnapshot>,
}

impl RequestExecution {
    /// 从执行结果构造（成功分支）
    pub fn success(
        id: String,
        request_id: String,
        started_at: i64,
        duration_ms: u64,
        response: ResponseSnapshot,
    ) -> Self {
        let status_code = Some(response.status);
        Self {
            id,
            request_id,
            started_at,
            duration_ms,
            status_code,
            success: (200..400).contains(&response.status),
            error: None,
            response: Some(response),
        }
    }

    /// 从执行结果构造（失败分支）
    pub fn failure(
        id: String,
        request_id: String,
        started_at: i64,
        duration_ms: u64,
        error: NetworkError,
    ) -> Self {
        Self {
            id,
            request_id,
            started_at,
            duration_ms,
            status_code: None,
            success: false,
            error: Some(error),
            response: None,
        }
    }
}

/// 历史列表项（带请求摘要，供前端列表展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionListItem {
    pub execution: RequestExecution,
    pub method: HttpMethod,
    pub url: String,
    pub name: String,
}
