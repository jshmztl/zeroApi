//! Repository 层（V2）
//!
//! 只负责数据持久化，不包含业务逻辑。
//! 业务编排在 `service/` 层完成。

mod collection_repo;
mod environment_repo;
mod execution_repo;
mod folder_repo;
mod project_repo;
mod request_repo;
mod settings_repo;

pub use collection_repo::*;
pub use environment_repo::*;
pub use execution_repo::*;
pub use folder_repo::*;
pub use project_repo::*;
pub use request_repo::*;
pub use settings_repo::*;

use std::sync::Arc;

use crate::db::Database;

/// Repository 集合（统一构造入口）
#[derive(Clone)]
pub struct Repos {
    pub projects: ProjectRepo,
    pub collections: CollectionRepo,
    pub folders: FolderRepo,
    pub requests: RequestRepo,
    pub executions: ExecutionRepo,
    pub environments: EnvironmentRepo,
    pub settings: SettingsRepo,
}

impl Repos {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            projects: ProjectRepo::new(db.clone()),
            collections: CollectionRepo::new(db.clone()),
            folders: FolderRepo::new(db.clone()),
            requests: RequestRepo::new(db.clone()),
            executions: ExecutionRepo::new(db.clone()),
            environments: EnvironmentRepo::new(db.clone()),
            settings: SettingsRepo::new(db.clone()),
        }
    }
}
