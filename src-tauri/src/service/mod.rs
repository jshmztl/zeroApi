//! Service 层（V2）
//!
//! 组织：Command → Service → Repository → SQLite
//! Service 组合 Repository 与 Transport，承载业务编排；不包含 SQL。

pub mod collection_service;
pub mod environment_service;
pub mod export_service;
pub mod history_service;
pub mod import_service;
pub mod openapi_service;
pub mod project_file;
pub mod project_service;
pub mod request_service;
pub mod settings_service;

use std::path::PathBuf;
use std::sync::Arc;

use crate::cancel;
use crate::db::Database;
use crate::repository::Repos;
use crate::transport::{HttpTransport, SessionManager};

pub use collection_service::*;
pub use environment_service::*;
pub use export_service::*;
pub use history_service::*;
pub use import_service::*;
pub use openapi_service::*;
pub use project_file::*;
pub use project_service::*;
pub use request_service::*;
pub use settings_service::*;

/// Service 集合（统一构造入口）
#[derive(Clone)]
pub struct Services {
    pub request: Arc<RequestService>,
    pub project: ProjectService,
    pub project_file: ProjectFileService,
    pub collection: CollectionService,
    pub environment: EnvironmentService,
    pub history: HistoryService,
    pub import: ImportService,
    pub export: ExportService,
    pub openapi: OpenApiService,
    pub settings: SettingsService,
}

impl Services {
    pub fn new(
        db: Arc<Database>,
        repos: &Repos,
        transport: Arc<HttpTransport>,
        sessions: Arc<SessionManager>,
        cancel_registry: cancel::SharedRegistry,
        responses_dir: PathBuf,
    ) -> Self {
        Self {
            request: Arc::new(RequestService::new(
                repos,
                transport.clone(),
                sessions.clone(),
                cancel_registry.clone(),
                responses_dir,
            )),
            project: ProjectService::new(repos),
            project_file: ProjectFileService::new(repos),
            collection: CollectionService::new(repos),
            environment: EnvironmentService::new(repos),
            history: HistoryService::new(repos),
            import: ImportService::new(db.clone(), repos),
            export: ExportService::new(repos),
            openapi: OpenApiService::new(db.clone(), repos),
            settings: SettingsService::new(repos),
        }
    }
}
