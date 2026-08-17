//! Project 文件格式（文档 §19-21）
//!
//! Git-friendly 目录结构：
//! ```text
//! my-project/
//! ├── zeroapi.yaml          # 项目元数据（version / project / settings）
//! ├── collections/*.yaml    # 每个集合一个文件（含其下请求）
//! └── environments/*.yaml   # 每个环境一个文件（Secret 只导出引用，不导出明文）
//! ```
//!
//! 原则（文档 §22）：人类可读、Git 可 diff、不保存 Secret / 历史 / 机器路径。

use serde::{Deserialize, Serialize};

use crate::domain::{AuthConfig, HeaderEntry, HttpMethod, KeyValue, RequestBody, VariableKind};

/// zeroapi.yaml 根结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFileRoot {
    pub version: u32,
    pub project: ProjectFileMeta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<ProjectFileSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFileMeta {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectFileSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_environment: Option<String>,
}

/// collections/<name>.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionFile {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requests: Vec<RequestFile>,
}

/// 请求文件条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestFile {
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    /// 所属 Folder 名（可选，导入时按名创建）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub query: Vec<KeyValue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<HeaderEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<RequestBody>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthConfig>,
}

/// environments/<name>.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentFile {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub base_url: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vars: Vec<VariableFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableFile {
    pub name: String,
    /// Secret 变量此值为空（只保留类型引用）
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub value: String,
    #[serde(default)]
    pub kind: VariableKind,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}
