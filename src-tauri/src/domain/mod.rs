//! V2 Domain Model
//!
//! 领域模型层：不依赖任何持久化 / 传输实现，只描述业务实体。
//!
//! 模块组织：
//! - `common`      通用类型（HttpMethod / KeyValue / HeaderEntry）
//! - `project`     Project（Git 管理边界）
//! - `collection`  Collection（Project 下的请求集合）
//! - `folder`      Folder（Collection 下的层级目录）
//! - `request`     Request / RequestBody / AuthConfig
//! - `execution`   RequestExecution（请求执行记录）
//! - `response`    ResponseSnapshot / ResponseBody
//! - `timing`      Timing（分段耗时）
//! - `error`       NetworkError / NetworkErrorKind（结构化网络错误）
//! - `environment` Environment / EnvironmentVariable / VariableKind
//! - `settings`    Settings

pub mod collection;
pub mod common;
pub mod environment;
pub mod error;
pub mod execution;
pub mod folder;
pub mod project;
pub mod request;
pub mod response;
pub mod settings;
pub mod timing;
pub mod transfer;

pub use collection::*;
pub use common::*;
pub use environment::*;
pub use error::*;
pub use execution::*;
pub use folder::*;
pub use project::*;
pub use request::*;
pub use response::*;
pub use settings::*;
pub use timing::*;
pub use transfer::*;
