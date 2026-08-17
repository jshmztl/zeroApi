//! Environment 服务

use uuid::Uuid;

use crate::domain::Environment;
use crate::repository::Repos;
use crate::AppResult;

#[derive(Clone)]
pub struct EnvironmentService {
    repos: Repos,
}

impl EnvironmentService {
    pub fn new(repos: &Repos) -> Self {
        Self {
            repos: repos.clone(),
        }
    }

    pub fn list(&self) -> AppResult<Vec<Environment>> {
        self.repos.environments.list()
    }

    pub fn get(&self, id: &str) -> AppResult<Option<Environment>> {
        self.repos.environments.get(id)
    }

    /// 保存环境（无 id 时生成）；返回环境 id
    pub fn save(&self, mut env: Environment) -> AppResult<String> {
        if env.id.is_empty() {
            env.id = Uuid::new_v4().to_string();
        }
        self.repos.environments.upsert(&env)?;
        Ok(env.id)
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        self.repos.environments.delete(id)
    }
}
