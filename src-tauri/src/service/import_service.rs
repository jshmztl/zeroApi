//! Import 服务：cURL / JSON

use std::sync::Arc;

use crate::db::Database;
use crate::domain::{ExportPayload, ImportPayload, Request};
use crate::repository::Repos;
use crate::AppResult;

#[derive(Clone)]
pub struct ImportService {
    db: Arc<Database>,
    repos: Repos,
}

impl ImportService {
    pub fn new(db: Arc<Database>, repos: &Repos) -> Self {
        Self {
            db,
            repos: repos.clone(),
        }
    }

    /// 解析 cURL 命令为 Request（V2 模型）
    pub fn import_curl(&self, command: &str) -> AppResult<Request> {
        crate::curl::parse(command)
    }

    /// 导入 JSON 备份（写库并返回负载）
    pub fn import_json(&self, content: &str) -> AppResult<ImportPayload> {
        let payload: ExportPayload = serde_json::from_str(content)
            .map_err(|e| crate::AppError::Other(format!("JSON 解析失败: {}", e)))?;

        for p in &payload.projects {
            self.repos.projects.upsert(p)?;
        }
        for c in &payload.collections {
            self.repos.collections.upsert(c)?;
        }
        for f in &payload.folders {
            self.repos.folders.upsert(f)?;
        }
        for r in &payload.requests {
            self.repos.requests.upsert(r)?;
        }
        for e in &payload.environments {
            self.repos.environments.upsert(e)?;
        }
        for f in &payload.favorites {
            self.repos.requests.add_favorite(f)?;
        }

        Ok(ImportPayload {
            projects: payload.projects,
            collections: payload.collections,
            folders: payload.folders,
            requests: payload.requests,
            environments: payload.environments,
            favorites: payload.favorites,
        })
    }

    // 持有 db 引用（备用：后续大文件导入走异步）
    #[allow(dead_code)]
    fn _db(&self) -> &Arc<Database> {
        &self.db
    }
}
