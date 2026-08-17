//! Environment 领域模型（V2）
//!
//! Secret 与普通变量分离：
//! - Plain：普通环境变量
//! - Secret：敏感变量（JWT / API Key / Password），默认不进入 Git 项目文件

use serde::{Deserialize, Serialize};

/// 变量类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableKind {
    Plain,
    Secret,
}

impl Default for VariableKind {
    fn default() -> Self {
        VariableKind::Plain
    }
}

/// 环境变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariable {
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub kind: VariableKind,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Secret 引用（内部使用，指向 env_secret_refs 表；不导出到项目文件）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_ref: Option<String>,
}

impl EnvironmentVariable {
    pub fn plain(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            kind: VariableKind::Plain,
            enabled: true,
            secret_ref: None,
        }
    }

    pub fn secret(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            kind: VariableKind::Secret,
            enabled: true,
            secret_ref: None,
        }
    }
}

/// 环境
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub vars: Vec<EnvironmentVariable>,
    #[serde(default)]
    pub active: bool,
}

impl Environment {
    /// 收集启用的变量（Plain + Secret 均参与请求编译）
    pub fn collect_vars(&self) -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        for v in &self.vars {
            if v.enabled && !v.name.is_empty() {
                m.insert(v.name.clone(), v.value.clone());
            }
        }
        if !self.base_url.is_empty() {
            m.insert("baseUrl".to_string(), self.base_url.clone());
            m.insert("base_url".to_string(), self.base_url.clone());
        }
        m
    }
}

fn default_true() -> bool {
    true
}
