//! 导入 / 导出容器（V2）

use serde::{Deserialize, Serialize};

use crate::domain::{Collection, Environment, Favorite, Folder, Project, Request};

/// 导出负载：可进入 Git 的完整项目快照（不含 Secret 明文值）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPayload {
    pub version: u32,
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub collections: Vec<Collection>,
    #[serde(default)]
    pub folders: Vec<Folder>,
    #[serde(default)]
    pub requests: Vec<Request>,
    #[serde(default)]
    pub environments: Vec<Environment>,
    #[serde(default)]
    pub favorites: Vec<Favorite>,
}

/// 导入负载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPayload {
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub collections: Vec<Collection>,
    #[serde(default)]
    pub folders: Vec<Folder>,
    #[serde(default)]
    pub requests: Vec<Request>,
    #[serde(default)]
    pub environments: Vec<Environment>,
    #[serde(default)]
    pub favorites: Vec<Favorite>,
}

impl ExportPayload {
    pub fn empty() -> Self {
        Self {
            version: 2,
            projects: Vec::new(),
            collections: Vec::new(),
            folders: Vec::new(),
            requests: Vec::new(),
            environments: Vec::new(),
            favorites: Vec::new(),
        }
    }
}
