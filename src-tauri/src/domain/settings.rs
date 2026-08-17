//! Settings 领域模型

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub proxy_url: String,
    /// light | dark | system
    #[serde(default)]
    pub theme: String,
    #[serde(default = "default_true")]
    pub auto_save_history: bool,
    #[serde(default = "default_history_limit")]
    pub history_limit: u32,
    #[serde(default = "default_true")]
    pub verify_ssl: bool,
    #[serde(default = "default_true")]
    pub follow_redirects: bool,
    /// 文本响应预览大小上限（字节），默认 5MB
    #[serde(default = "default_max_preview")]
    pub max_preview_size: u64,
}

fn default_timeout() -> u64 {
    30000
}
fn default_true() -> bool {
    true
}
fn default_history_limit() -> u32 {
    100
}
fn default_max_preview() -> u64 {
    5 * 1024 * 1024
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            timeout_ms: 30000,
            proxy_url: String::new(),
            theme: "light".to_string(),
            auto_save_history: true,
            history_limit: 100,
            verify_ssl: true,
            follow_redirects: true,
            max_preview_size: 5 * 1024 * 1024,
        }
    }
}
