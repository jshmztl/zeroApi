//! Request 服务：发送请求、执行记录、收藏、保存

use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::cancel;
use crate::domain::*;
use crate::repository::{CollectionRepo, EnvironmentRepo, ExecutionRepo, Repos, RequestRepo, SettingsRepo};
use crate::transport::compiler::{compile, CompileContext};
use crate::transport::{HttpTransport, SessionManager};
use crate::AppResult;

pub struct RequestService {
    requests: RequestRepo,
    executions: ExecutionRepo,
    environments: EnvironmentRepo,
    collections: CollectionRepo,
    settings: SettingsRepo,
    transport: Arc<HttpTransport>,
    sessions: Arc<SessionManager>,
    cancel_registry: cancel::SharedRegistry,
    responses_dir: PathBuf,
}

impl RequestService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        repos: &Repos,
        transport: Arc<HttpTransport>,
        sessions: Arc<SessionManager>,
        cancel_registry: cancel::SharedRegistry,
        responses_dir: PathBuf,
    ) -> Self {
        Self {
            requests: repos.requests.clone(),
            executions: repos.executions.clone(),
            environments: repos.environments.clone(),
            collections: repos.collections.clone(),
            settings: repos.settings.clone(),
            transport,
            sessions,
            cancel_registry,
            responses_dir,
        }
    }

    /// 发送请求（编译 → 传输 → 记录执行）
    ///
    /// 取消语义（文档 §30）：取消信号触发时底层 HTTP Future 被真正终止，
    /// 并产生一条 Cancelled 执行记录。
    pub async fn send(
        &self,
        request: &Request,
        client_id: Option<String>,
    ) -> AppResult<ResponseSnapshot> {
        let settings = self.settings.get()?;
        let envs = self.environments.list()?;
        let active = envs.iter().find(|e| e.active);
        let ctx = CompileContext {
            env: active,
            settings: &settings,
        };
        let compiled = compile(request, &ctx)?;

        // Cookie 隔离：按请求所属 Project 取 Session
        let project_id = self
            .collections
            .get(&request.collection_id)?
            .map(|c| c.project_id)
            .unwrap_or_else(|| DEFAULT_PROJECT_ID.to_string());
        let session = self.sessions.get(&project_id, &self.transport).await?;

        // 取消注册
        let cid = client_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        self.cancel_registry.0.lock().await.insert(cid.clone(), tx);

        let started_at = Utc::now().timestamp_millis();
        let start = std::time::Instant::now();
        let execute = async {
            let r = self
                .transport
                .execute(&session.client, &compiled, settings.timeout_ms, &self.responses_dir)
                .await;
            (r, start.elapsed().as_millis() as u64)
        };

        let outcome = tokio::select! {
            (r, dur) = execute => r.map(|resp| (resp, dur)),
            _ = rx => Err(NetworkError::new(NetworkErrorKind::Cancelled, "请求已取消")),
        };
        self.cancel_registry.0.lock().await.remove(&cid);

        match outcome {
            Ok((resp, dur)) => {
                let exec = RequestExecution::success(
                    Uuid::new_v4().to_string(),
                    request.id.clone(),
                    started_at,
                    dur,
                    resp.clone(),
                );
                if settings.auto_save_history {
                    self.record_execution(&exec, request, &settings);
                }
                Ok(resp)
            }
            Err(ne) => {
                let exec = RequestExecution::failure(
                    Uuid::new_v4().to_string(),
                    request.id.clone(),
                    started_at,
                    0,
                    ne.clone(),
                );
                if settings.auto_save_history {
                    self.record_execution(&exec, request, &settings);
                }
                Err(ne.into())
            }
        }
    }

    /// 记录执行并裁剪超限历史
    fn record_execution(&self, exec: &RequestExecution, request: &Request, settings: &Settings) {
        let r = self
            .executions
            .insert(exec, request.method.as_str(), &request.url)
            .and_then(|_| self.executions.prune(settings.history_limit.max(1)));
        if let Err(e) = r {
            log::error!("保存执行记录失败: {}", e);
        }
    }

    // ---------- Request CRUD ----------

    pub fn list_requests(
        &self,
        collection_id: Option<String>,
        folder_id: Option<String>,
    ) -> AppResult<Vec<Request>> {
        self.requests.list(collection_id.as_deref(), folder_id.as_deref())
    }

    pub fn get_request(&self, id: &str) -> AppResult<Option<Request>> {
        self.requests.get(id)
    }

    /// 保存请求（无 id 时生成）；返回请求 id
    pub fn save_request(&self, mut request: Request) -> AppResult<String> {
        let now = Utc::now().timestamp_millis();
        if request.id.is_empty() {
            request.id = Uuid::new_v4().to_string();
        }
        if request.created_at == 0 {
            request.created_at = now;
        }
        request.updated_at = now;
        if request.collection_id.is_empty() {
            request.collection_id = crate::db::UNCATEGORIZED_COLLECTION.to_string();
        }
        self.requests.upsert(&request)?;
        Ok(request.id)
    }

    pub fn delete_request(&self, id: &str) -> AppResult<()> {
        self.requests.delete(id)
    }

    // ---------- 收藏 ----------

    pub fn add_favorite(&self, mut request: Request) -> AppResult<String> {
        let now = Utc::now().timestamp_millis();
        if request.id.is_empty() {
            request.id = Uuid::new_v4().to_string();
        }
        let fav = Favorite {
            id: Uuid::new_v4().to_string(),
            request,
            created_at: now,
        };
        self.requests.add_favorite(&fav)?;
        Ok(fav.id)
    }

    pub fn remove_favorite(&self, id: &str) -> AppResult<()> {
        self.requests.remove_favorite(id)
    }

    pub fn list_favorites(&self) -> AppResult<Vec<Favorite>> {
        self.requests.list_favorites()
    }
}
