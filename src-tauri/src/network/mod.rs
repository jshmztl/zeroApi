//! Network Diagnostic（文档 §27-29）
//!
//! 目录：
//! - `dns`        DNS 解析探测
//! - `tcp`        TCP 连接探测
//! - `tls`        TLS 握手探测
//! - `diagnostic` 组合诊断（DNS → TCP → TLS → HTTP）

pub mod diagnostic;
pub mod dns;
pub mod tcp;
pub mod tls;
pub mod proxy;

pub use diagnostic::{diagnose, DiagnosticResult, HttpProbeResult};
pub use dns::{resolve, DnsResult};
pub use tcp::{connect, TcpResult};
pub use tls::{handshake, TlsResult};
pub use proxy::{diagnose_proxy, ProxyProbeResult};
