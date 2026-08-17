//! Project 服务

use chrono::Utc;
use uuid::Uuid;

use crate::domain::Project;
use crate::repository::Repos;
use crate::AppResult;

#[derive(Clone)]
pub struct ProjectService {
    repos: Repos,
}

impl ProjectService {
    pub fn new(repos: &Repos) -> Self {
        Self {
            repos: repos.clone(),
        }
    }

    pub fn list(&self) -> AppResult<Vec<Project>> {
        self.repos.projects.list()
    }

    pub fn get(&self, id: &str) -> AppResult<Option<Project>> {
        self.repos.projects.get(id)
    }

    /// 保存项目（无 id 时生成）；返回项目 id
    pub fn save(&self, mut project: Project) -> AppResult<String> {
        let now = Utc::now().timestamp_millis();
        if project.id.is_empty() {
            project.id = Uuid::new_v4().to_string();
        }
        if project.created_at == 0 {
            project.created_at = now;
        }
        project.updated_at = now;
        self.repos.projects.upsert(&project)?;
        Ok(project.id)
    }

    /// 删除项目（级联删除其集合 / 请求 / 执行记录）
    pub fn delete(&self, id: &str) -> AppResult<()> {
        self.repos.projects.delete(id)
    }
}
