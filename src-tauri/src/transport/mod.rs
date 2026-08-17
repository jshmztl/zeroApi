//! Transport 层（V2）
//!
//! 请求编译与传输解耦：
//! - `compiler`：RequestCompiler（Request + Environment + Secret + Settings → CompiledRequest）
//! - `http`：HttpTransport（CompiledRequest → ResponseSnapshot）
//! - `session`：Cookie Session 隔离（按 Project）
//!
//! V2 初期只实现 HTTP；WebSocket / gRPC 仅保留接口占位（见文档 §13）。

pub mod compiler;
pub mod http;
pub mod session;

pub use compiler::{compile, CompileContext, CompiledBody, CompiledRequest};
pub use http::HttpTransport;
pub use session::{Session, SessionManager};

/// WebSocket 传输（V2 暂不实现，接口占位）
pub mod websocket {
    //! WebSocket Transport —— 预留，V2 不实现（见 ZEROAPI_V2_IMPLEMENTATION.md §13 / §36）
}

/// gRPC 传输（V2 暂不实现，接口占位）
pub mod grpc {
    //! gRPC Transport —— 预留，V2 不实现（见 ZEROAPI_V2_IMPLEMENTATION.md §13 / §36）
}
