//! Project File 服务（Git-friendly 项目文件导入 / 导出，文档 §19-22）

use std::collections::HashMap;
use std::path::Path;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::project_file::*;
use crate::domain::*;
use crate::repository::Repos;
use crate::AppError;
use crate::AppResult;

/// 项目文件名
const PROJECT_FILE_NAME: &str = "zeroapi.yaml";

#[derive(Clone)]
pub struct ProjectFileService {
    repos: Repos,
}

impl ProjectFileService {
    pub fn new(repos: &Repos) -> Self {
        Self {
            repos: repos.clone(),
        }
    }

    // ==================== 导出 ====================

    /// 导出项目到目录（可进入 Git）
    pub fn export_project(&self, project_id: &str, dir: &Path) -> AppResult<()> {
        let project = self
            .repos
            .projects
            .get(project_id)?
            .ok_or_else(|| AppError::Other("项目不存在".into()))?;

        let collections = self.repos.collections.list(Some(project_id))?;
        let all_requests = self.repos.requests.list(None, None)?;
        let all_folders = self.repos.folders.list(None)?;
        let envs: Vec<Environment> = self
            .repos
            .environments
            .list()?
            .into_iter()
            .filter(|e| e.project_id.as_deref() == Some(project_id))
            .collect();

        // 重建目录（保证导出结果干净）
        if dir.exists() {
            std::fs::remove_dir_all(dir)?;
        }
        std::fs::create_dir_all(dir.join("collections"))?;
        std::fs::create_dir_all(dir.join("environments"))?;

        // ---- zeroapi.yaml ----
        let root = ProjectFileRoot {
            version: 1,
            project: ProjectFileMeta {
                name: project.name.clone(),
                description: project.description.clone(),
            },
            settings: Some(ProjectFileSettings {
                default_environment: envs.iter().find(|e| e.active).map(|e| e.name.clone()),
            }),
        };
        let yaml = serde_yml::to_string(&root).map_err(|e| AppError::Other(format!("YAML 序列化失败: {}", e)))?;
        std::fs::write(dir.join(PROJECT_FILE_NAME), yaml)?;

        // ---- collections/*.yaml ----
        let folder_name_by_id: HashMap<String, String> = all_folders
            .iter()
            .map(|f| (f.id.clone(), f.name.clone()))
            .collect();
        let mut used_names: HashMap<String, usize> = HashMap::new();
        for col in &collections {
            let reqs: Vec<&Request> = all_requests
                .iter()
                .filter(|r| r.collection_id == col.id)
                .collect();
            let cf = CollectionFile {
                name: col.name.clone(),
                description: col.description.clone(),
                requests: reqs
                    .iter()
                    .map(|r| RequestFile {
                        name: r.name.clone(),
                        method: r.method,
                        url: r.url.clone(),
                        folder: r
                            .folder_id
                            .as_deref()
                            .and_then(|fid| folder_name_by_id.get(fid))
                            .cloned(),
                        query: r.query.clone(),
                        headers: r.headers.clone(),
                        body: r.body.clone(),
                        auth: r.auth.clone(),
                    })
                    .collect(),
            };
            let yaml = serde_yml::to_string(&cf)
                .map_err(|e| AppError::Other(format!("YAML 序列化失败: {}", e)))?;
            let file_name = unique_slug(&col.name, "yaml", &mut used_names);
            std::fs::write(dir.join("collections").join(file_name), yaml)?;
        }

        // ---- environments/*.yaml（Secret 只导出引用）----
        let mut used_env_names: HashMap<String, usize> = HashMap::new();
        for env in &envs {
            let ef = EnvironmentFile {
                name: env.name.clone(),
                base_url: env.base_url.clone(),
                vars: env
                    .vars
                    .iter()
                    .map(|v| VariableFile {
                        name: v.name.clone(),
                        // Secret 明文绝不进入项目文件
                        value: if v.kind == VariableKind::Secret {
                            String::new()
                        } else {
                            v.value.clone()
                        },
                        kind: v.kind,
                        enabled: v.enabled,
                    })
                    .collect(),
            };
            let yaml = serde_yml::to_string(&ef)
                .map_err(|e| AppError::Other(format!("YAML 序列化失败: {}", e)))?;
            let file_name = unique_slug(&env.name, "yaml", &mut used_env_names);
            std::fs::write(dir.join("environments").join(file_name), yaml)?;
        }

        Ok(())
    }

    // ==================== 导入 ====================

    /// 从目录导入项目（新建项目 + 集合 + 请求 + 环境）；返回项目 id
    pub fn import_project(&self, dir: &Path) -> AppResult<String> {
        let root_path = dir.join(PROJECT_FILE_NAME);
        if !root_path.exists() {
            return Err(AppError::Other(format!(
                "目录中缺少 {}（不是 ZeroApi 项目文件）",
                PROJECT_FILE_NAME
            )));
        }
        let raw = std::fs::read_to_string(&root_path)?;
        let root: ProjectFileRoot = serde_yml::from_str(&raw)
            .map_err(|e| AppError::Other(format!("解析 {} 失败: {}", PROJECT_FILE_NAME, e)))?;

        let now = Utc::now().timestamp_millis();
        let project = Project {
            id: Uuid::new_v4().to_string(),
            name: root.project.name,
            description: root.project.description,
            root_path: Some(dir.to_string_lossy().into_owned()),
            created_at: now,
            updated_at: now,
        };
        self.repos.projects.upsert(&project)?;

        // ---- collections/*.yaml ----
        let collections_dir = dir.join("collections");
        if collections_dir.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(&collections_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "yaml" || x == "yml").unwrap_or(false))
                .collect();
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                let cf: CollectionFile = serde_yml::from_str(&std::fs::read_to_string(entry.path())?)
                    .map_err(|e| AppError::Other(format!("解析集合文件失败: {}", e)))?;
                self.import_collection(&project.id, &cf, now)?;
            }
        }

        // ---- environments/*.yaml ----
        let env_dir = dir.join("environments");
        if env_dir.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(&env_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "yaml" || x == "yml").unwrap_or(false))
                .collect();
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                let ef: EnvironmentFile = serde_yml::from_str(&std::fs::read_to_string(entry.path())?)
                    .map_err(|e| AppError::Other(format!("解析环境文件失败: {}", e)))?;
                self.import_environment(&project.id, &ef, now)?;
            }
        }

        Ok(project.id)
    }

    fn import_collection(&self, project_id: &str, cf: &CollectionFile, now: i64) -> AppResult<()> {
        let col = Collection {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            name: cf.name.clone(),
            description: cf.description.clone(),
            sort_order: 0,
            created_at: now,
            updated_at: now,
        };
        self.repos.collections.upsert(&col)?;

        // folder 名 → id（导入时按名创建）
        let mut folder_ids: HashMap<String, String> = HashMap::new();
        for req in &cf.requests {
            let folder_id = match &req.folder {
                Some(fname) => {
                    if let Some(fid) = folder_ids.get(fname) {
                        Some(fid.clone())
                    } else {
                        let fid = Uuid::new_v4().to_string();
                        self.repos.folders.upsert(&Folder {
                            id: fid.clone(),
                            collection_id: col.id.clone(),
                            parent_id: None,
                            name: fname.clone(),
                            sort_order: 0,
                        })?;
                        folder_ids.insert(fname.clone(), fid.clone());
                        Some(fid)
                    }
                }
                None => None,
            };
            let request = Request {
                id: Uuid::new_v4().to_string(),
                collection_id: col.id.clone(),
                folder_id,
                name: req.name.clone(),
                method: req.method,
                url: req.url.clone(),
                headers: req.headers.clone(),
                query: req.query.clone(),
                body: req.body.clone(),
                auth: req.auth.clone(),
                sort_order: 0,
                created_at: now,
                updated_at: now,
            };
            self.repos.requests.upsert(&request)?;
        }
        Ok(())
    }

    fn import_environment(&self, project_id: &str, ef: &EnvironmentFile, _now: i64) -> AppResult<()> {
        let env = Environment {
            id: Uuid::new_v4().to_string(),
            project_id: Some(project_id.to_string()),
            name: ef.name.clone(),
            base_url: ef.base_url.clone(),
            vars: ef
                .vars
                .iter()
                .map(|v| EnvironmentVariable {
                    name: v.name.clone(),
                    // Secret 导入时无明文，值为空（待用户填写）
                    value: v.value.clone(),
                    kind: v.kind,
                    enabled: v.enabled,
                    secret_ref: None,
                })
                .collect(),
            active: false,
        };
        self.repos.environments.upsert(&env)?;
        Ok(())
    }
}

/// 生成唯一文件名（slug + 序号去重）
fn unique_slug(name: &str, ext: &str, used: &mut HashMap<String, usize>) -> String {
    let base: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    let base = if base.is_empty() { "untitled".to_string() } else { base };
    let count = used.entry(base.clone()).or_insert(0);
    *count += 1;
    if *count == 1 {
        format!("{}.{}", base, ext)
    } else {
        format!("{}_{}.{}", base, *count - 1, ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::repository::Repos;
    use std::sync::Arc;

    fn setup() -> (Arc<Database>, Repos) {
        let dir = std::env::temp_dir().join(format!("zeroapi-pf-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = Arc::new(Database::new(dir.join("test.db")).unwrap());
        db.migrate().unwrap();
        let repos = Repos::new(db.clone());
        (db, repos)
    }

    /// 导出 → 导入 round-trip：数据一致，Secret 明文不出现在项目文件中
    #[test]
    fn test_export_import_roundtrip() {
        let (_db, repos) = setup();
        let svc = ProjectFileService::new(&repos);
        let now = chrono::Utc::now().timestamp_millis();

        // 构造项目数据
        let project = Project {
            id: "p1".into(),
            name: "Demo API".into(),
            description: Some("示例项目".into()),
            root_path: None,
            created_at: now,
            updated_at: now,
        };
        repos.projects.upsert(&project).unwrap();
        let col = Collection {
            id: "c1".into(),
            project_id: "p1".into(),
            name: "用户".into(),
            description: None,
            sort_order: 0,
            created_at: now,
            updated_at: now,
        };
        repos.collections.upsert(&col).unwrap();
        let req = Request {
            id: "r1".into(),
            collection_id: "c1".into(),
            folder_id: None,
            name: "获取用户".into(),
            method: HttpMethod::Get,
            url: "{{base_url}}/users".into(),
            headers: vec![HeaderEntry::new("Accept", "application/json")],
            query: vec![KeyValue::new("page", "1")],
            body: None,
            auth: None,
            sort_order: 0,
            created_at: now,
            updated_at: now,
        };
        repos.requests.upsert(&req).unwrap();
        // 环境含一个 Secret 变量（upsert 时会加密存储）
        let env = Environment {
            id: "e1".into(),
            project_id: Some("p1".into()),
            name: "开发".into(),
            base_url: "https://dev.example.com".into(),
            vars: vec![
                EnvironmentVariable::plain("TOKEN", "sk-plain"),
                EnvironmentVariable::secret("API_KEY", "sk-secret-123"),
            ],
            active: true,
        };
        repos.environments.upsert(&env).unwrap();

        // 导出到临时目录
        let out_dir = std::env::temp_dir().join(format!("zeroapi-export-{}", uuid::Uuid::new_v4().simple()));
        svc.export_project("p1", &out_dir).unwrap();
        assert!(out_dir.join("zeroapi.yaml").exists());

        // 扫描导出目录所有文件，Secret 明文绝不允许出现
        let mut all_text = String::new();
        fn collect(dir: &std::path::Path, acc: &mut String) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        collect(&p, acc);
                    } else if p.extension().map(|x| x == "yaml" || x == "yml").unwrap_or(false) {
                        acc.push_str(&std::fs::read_to_string(&p).unwrap_or_default());
                    }
                }
            }
        }
        collect(&out_dir, &mut all_text);
        assert!(!all_text.contains("sk-secret-123"), "Secret 明文泄漏到项目文件!");
        assert!(all_text.contains("API_KEY"), "Secret 变量名应保留");
        assert!(all_text.contains("sk-plain"), "普通变量应保留");

        // 导入
        let new_pid = svc.import_project(&out_dir).unwrap();
        let projects = repos.projects.list().unwrap();
        assert!(projects.iter().any(|p| p.id == new_pid && p.name == "Demo API"));

        let envs = repos.environments.list().unwrap();
        let imported_env = envs
            .iter()
            .find(|e| e.project_id.as_deref() == Some(new_pid.as_str()))
            .expect("导入后应有环境");
        assert_eq!(imported_env.name, "开发");
        assert_eq!(imported_env.base_url, "https://dev.example.com");
        let api_key = imported_env.vars.iter().find(|v| v.name == "API_KEY").unwrap();
        assert_eq!(api_key.kind, VariableKind::Secret);
        assert!(api_key.value.is_empty(), "导入的 Secret 值应为空（待重新填写）");

        let reqs = repos.requests.list(None, None).unwrap();
        assert!(
            reqs.iter()
                .any(|r| r.name == "获取用户" && r.url == "{{base_url}}/users" && r.collection_id != "c1")
        );

        std::fs::remove_dir_all(&out_dir).ok();
    }
}
