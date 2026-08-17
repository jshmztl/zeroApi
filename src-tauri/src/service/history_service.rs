//! History 服务（基于 RequestExecution）

use crate::domain::{ExecutionListItem, RequestExecution};
use crate::repository::Repos;
use crate::AppResult;

#[derive(Clone)]
pub struct HistoryService {
    repos: Repos,
}

impl HistoryService {
    pub fn new(repos: &Repos) -> Self {
        Self {
            repos: repos.clone(),
        }
    }

    /// 历史列表（带请求摘要）
    pub fn list(&self, limit: u32) -> AppResult<Vec<ExecutionListItem>> {
        self.repos.executions.list_summary(limit.max(1))
    }

    pub fn get(&self, id: &str) -> AppResult<Option<RequestExecution>> {
        self.repos.executions.get(id)
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        self.repos.executions.delete(id)
    }

    pub fn clear(&self) -> AppResult<()> {
        self.repos.executions.clear()
    }
}
