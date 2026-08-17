//! Folder 领域模型
//!
//! Folder 挂在 Collection 下，支持多级（parent_id 指向另一 Folder）。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub collection_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub sort_order: i32,
}
