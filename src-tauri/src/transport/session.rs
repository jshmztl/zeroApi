//! Cookie Session 隔离（文档 §31）
//!
//! 目标：避免所有 Project 共享 Cookie。
//! 每个 Project 拥有独立的 `Session`（CookieJar + 缓存的 reqwest Client）。

use std::collections::HashMap;
use std::sync::Arc;

use reqwest::cookie::CookieStore;
use tokio::sync::Mutex;

use crate::transport::HttpTransport;

/// 一次会话：独立 CookieJar + 缓存的 HTTP Client
pub struct Session {
    pub jar: Arc<reqwest::cookie::Jar>,
    pub client: reqwest::Client,
}

/// 会话管理器：project_id → Session
pub struct SessionManager {
    sessions: Mutex<HashMap<String, Arc<Session>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// 获取（或创建）指定 Project 的会话
    pub async fn get(&self, project_id: &str, transport: &HttpTransport) -> Arc<Session> {
        let mut map = self.sessions.lock().await;
        if let Some(s) = map.get(project_id) {
            return s.clone();
        }
        let jar = Arc::new(reqwest::cookie::Jar::default());
        let client = transport
            .build_client(Some(jar.clone()))
            .expect("构建 HTTP Client 失败");
        let session = Arc::new(Session { jar, client });
        map.insert(project_id.to_string(), session.clone());
        session
    }

    /// 清除指定 Project 的会话（Cookie 失效）
    pub async fn clear(&self, project_id: &str) {
        self.sessions.lock().await.remove(project_id);
    }

    /// 清空全部会话
    pub async fn clear_all(&self) {
        self.sessions.lock().await.clear();
    }

    /// 查询指定 URL 域下的 Cookie（返回 "k=v; k2=v2"，无则空串）
    pub async fn cookies_for(&self, project_id: &str, url: &str) -> String {
        let map = self.sessions.lock().await;
        match map.get(project_id) {
            Some(s) => {
                if let Ok(u) = url::Url::parse(url) {
                    s.jar
                        .cookies(&u)
                        .map(|v| v.to_str().unwrap_or("").to_string())
                        .unwrap_or_default()
                } else {
                    String::new()
                }
            }
            None => String::new(),
        }
    }

    /// 会话数量（诊断用）
    pub async fn count(&self) -> usize {
        self.sessions.lock().await.len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
