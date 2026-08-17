//! Collection / Folder 服务

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{Collection, Folder};
use crate::repository::Repos;
use crate::AppResult;

#[derive(Clone)]
pub struct CollectionService {
    repos: Repos,
}

impl CollectionService {
    pub fn new(repos: &Repos) -> Self {
        Self {
            repos: repos.clone(),
        }
    }

    // ---------- Collection ----------

    pub fn list(&self, project_id: Option<String>) -> AppResult<Vec<Collection>> {
        self.repos.collections.list(project_id.as_deref())
    }

    pub fn get(&self, id: &str) -> AppResult<Option<Collection>> {
        self.repos.collections.get(id)
    }

    /// 保存集合（无 id 时生成）；返回集合 id
    pub fn save(&self, mut collection: Collection) -> AppResult<String> {
        let now = Utc::now().timestamp_millis();
        if collection.id.is_empty() {
            collection.id = Uuid::new_v4().to_string();
        }
        if collection.project_id.is_empty() {
            collection.project_id = crate::domain::DEFAULT_PROJECT_ID.to_string();
        }
        if collection.created_at == 0 {
            collection.created_at = now;
        }
        collection.updated_at = now;
        self.repos.collections.upsert(&collection)?;
        Ok(collection.id)
    }

    /// 删除集合（级联删除其 Folder / Request / 执行记录）
    pub fn delete(&self, id: &str) -> AppResult<()> {
        self.repos.collections.delete(id)
    }

    // ---------- Folder ----------

    pub fn list_folders(&self, collection_id: Option<String>) -> AppResult<Vec<Folder>> {
        self.repos.folders.list(collection_id.as_deref())
    }

    /// 保存 Folder（无 id 时生成）；返回 id
    pub fn save_folder(&self, mut folder: Folder) -> AppResult<String> {
        if folder.id.is_empty() {
            folder.id = Uuid::new_v4().to_string();
        }
        self.repos.folders.upsert(&folder)?;
        Ok(folder.id)
    }

    pub fn delete_folder(&self, id: &str) -> AppResult<()> {
        self.repos.folders.delete(id)
    }
}
