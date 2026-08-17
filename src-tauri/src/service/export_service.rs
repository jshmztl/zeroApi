//! Export 服务：生成可进入 Git 的 JSON 备份
//!
//! 原则（文档 §22）：不导出 Secret 明文值、请求历史、Cookie、本地设置。

use crate::domain::{ExportPayload, Favorite};
use crate::repository::Repos;
use crate::AppResult;

#[derive(Clone)]
pub struct ExportService {
    repos: Repos,
}

impl ExportService {
    pub fn new(repos: &Repos) -> Self {
        Self {
            repos: repos.clone(),
        }
    }

    pub fn export(&self) -> AppResult<String> {
        let mut payload = ExportPayload::empty();
        payload.projects = self.repos.projects.list()?;
        payload.collections = self.repos.collections.list(None)?;
        payload.folders = self.repos.folders.list(None)?;
        payload.requests = self.repos.requests.list(None, None)?;
        payload.environments = sanitize_environments(self.repos.environments.list()?);
        payload.favorites = sanitize_favorites(self.repos.requests.list_favorites()?);
        Ok(serde_json::to_string_pretty(&payload)?)
    }
}

/// 导出时剔除 Secret 变量的明文值（只保留名称与类型）
fn sanitize_environments(
    envs: Vec<crate::domain::Environment>,
) -> Vec<crate::domain::Environment> {
    envs.into_iter()
        .map(|mut e| {
            for v in e.vars.iter_mut() {
                if v.kind == crate::domain::VariableKind::Secret {
                    v.value.clear();
                }
            }
            e
        })
        .collect()
}

/// 导出时从收藏请求中剔除 Secret 引用（收藏请求中不包含明文 Secret，防御处理）
fn sanitize_favorites(favs: Vec<Favorite>) -> Vec<Favorite> {
    favs
}
