//! Timing 领域模型（分段耗时）

use serde::{Deserialize, Serialize};

/// 请求分段耗时
///
/// 各阶段为 Option：无法测量的阶段为 None。
/// Phase 1 保证 total_ms 精确；DNS/TCP/TLS 分段由
/// Network Diagnostic（Phase 4）完整实现。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Timing {
    pub dns_ms: Option<u64>,
    pub tcp_ms: Option<u64>,
    pub tls_ms: Option<u64>,
    /// 请求发送到首字节
    pub request_ms: Option<u64>,
    /// 首字节到响应完成
    pub response_ms: Option<u64>,
    pub total_ms: u64,
}

impl Timing {
    pub fn total(total_ms: u64) -> Self {
        Self {
            dns_ms: None,
            tcp_ms: None,
            tls_ms: None,
            request_ms: None,
            response_ms: None,
            total_ms,
        }
    }
}
