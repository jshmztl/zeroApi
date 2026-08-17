//! Project 领域模型
//!
//! Project 是：
//! - Git 管理边界
//! - Environment 管理边界
//! - Collection 管理边界
//! - Secret 引用边界

use serde::{Deserialize, Serialize};

/// 默认项目 ID（V1 数据迁移后的归属项目）
pub const DEFAULT_PROJECT_ID: &str = "default";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_path: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
