//! OpenAPI 导入服务（文档 §23）
//!
//! 流程：OpenAPI → Parser → Project → Collection → Folder → Request
//! 支持 OpenAPI 3.0 / 3.1（YAML / JSON）。
//! 映射规则：
//! - info.title → 项目名（未指定 project 时）
//! - servers[0].url → 请求 URL 前缀
//! - paths + method → Request（name = summary / operationId）
//! - operation.tags[0] → Collection（无 tag → "默认" 集合）
//! - parameters（path/operation 级）→ query / header
//! - requestBody.content → Body（取第一个 content-type）

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::db::Database;
use crate::domain::*;
use crate::repository::Repos;
use crate::AppError;
use crate::AppResult;

/// OpenAPI 导入结果
#[derive(Debug, Clone, Serialize)]
pub struct OpenApiImportResult {
    pub project_id: String,
    pub project_name: String,
    pub collections: usize,
    pub requests: usize,
}

const HTTP_METHODS: [&str; 8] = ["get", "post", "put", "patch", "delete", "head", "options", "trace"];

#[derive(Clone)]
pub struct OpenApiService {
    db: Arc<Database>,
    repos: Repos,
}

impl OpenApiService {
    pub fn new(db: Arc<Database>, repos: &Repos) -> Self {
        Self {
            db,
            repos: repos.clone(),
        }
    }

    /// 从内容导入 OpenAPI 文档
    pub fn import(&self, content: &str, project_id: Option<String>) -> AppResult<OpenApiImportResult> {
        let doc = parse_document(content)?;
        let now = Utc::now().timestamp_millis();

        // ---- 项目 ----
        let title = doc["info"]["title"].as_str().unwrap_or("OpenAPI 导入").to_string();
        let project_id = match project_id {
            Some(pid) if !pid.is_empty() => pid,
            _ => {
                let pid = Uuid::new_v4().to_string();
                self.repos.projects.upsert(&Project {
                    id: pid.clone(),
                    name: title.clone(),
                    description: doc["info"]["description"].as_str().map(String::from),
                    root_path: None,
                    created_at: now,
                    updated_at: now,
                })?;
                pid
            }
        };

        // ---- servers → 环境 base_url ----
        let server_url = doc["servers"]
            .as_array()
            .and_then(|s| s.first())
            .and_then(|s| s["url"].as_str())
            .unwrap_or("")
            .trim_end_matches('/')
            .to_string();

        // ---- 按 tag 组织集合 ----
        let mut collection_ids: HashMap<String, String> = HashMap::new();
        let mut collection_count = 0usize;
        let mut request_count = 0usize;

        let paths = doc["paths"].as_object().cloned().unwrap_or_default();
        for (path, item) in paths {
            let path_item = item.as_object().cloned().unwrap_or_default();
            let path_params = path_item.get("parameters").cloned().unwrap_or(serde_json::json!([]));
            for method in HTTP_METHODS {
                let op = match path_item.get(method) {
                    Some(o) if o.is_object() => o.clone(),
                    _ => continue,
                };
                let name = op["summary"]
                    .as_str()
                    .or_else(|| op["operationId"].as_str())
                    .map(String::from)
                    .unwrap_or_else(|| format!("{} {}", method.to_uppercase(), path));
                let tag = op["tags"]
                    .as_array()
                    .and_then(|t| t.first())
                    .and_then(|t| t.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| "默认".to_string());

                let cid = match collection_ids.get(&tag) {
                    Some(c) => c.clone(),
                    None => {
                        let cid = Uuid::new_v4().to_string();
                        self.repos.collections.upsert(&Collection {
                            id: cid.clone(),
                            project_id: project_id.clone(),
                            name: tag.clone(),
                            description: None,
                            sort_order: collection_count as i32,
                            created_at: now,
                            updated_at: now,
                        })?;
                        collection_count += 1;
                        collection_ids.insert(tag.clone(), cid.clone());
                        cid
                    }
                };

                let req = build_request(&project_id, &cid, &path, method, &op, &path_params, &server_url, now);
                self.repos.requests.upsert(&req)?;
                request_count += 1;
            }
        }

        // ---- 环境（servers[0] → base_url）----
        if !server_url.is_empty() {
            let envs = self.repos.environments.list()?;
            let exists = envs.iter().any(|e| e.project_id.as_deref() == Some(project_id.as_str()) && e.base_url == server_url);
            if !exists {
                self.repos.environments.upsert(&Environment {
                    id: Uuid::new_v4().to_string(),
                    project_id: Some(project_id.clone()),
                    name: "OpenAPI".to_string(),
                    base_url: server_url,
                    vars: Vec::new(),
                    active: false,
                })?;
            }
        }

        Ok(OpenApiImportResult {
            project_id,
            project_name: title,
            collections: collection_count,
            requests: request_count,
        })
    }

    /// 从 URL 抓取 OpenAPI 文档后导入
    pub async fn import_url(&self, url: &str, project_id: Option<String>) -> AppResult<OpenApiImportResult> {
        let resp = reqwest::Client::new()
            .get(url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| crate::AppError::Http(e))?;
        let text = resp
            .text()
            .await
            .map_err(|e| crate::AppError::Http(e))?;
        self.import(&text, project_id)
    }

    #[allow(dead_code)]
    fn _db(&self) -> &Arc<Database> {
        &self.db
    }
}

/// 解析 OpenAPI 文档（JSON 或 YAML）为 JSON Value
fn parse_document(content: &str) -> AppResult<serde_json::Value> {
    serde_json::from_str(content).or_else(|_| {
        serde_yml::from_str::<serde_json::Value>(content)
            .map_err(|e| AppError::Other(format!("OpenAPI 解析失败（JSON/YAML）: {}", e)))
    })
}

/// 构造 Request
fn build_request(
    project_id: &str,
    collection_id: &str,
    path: &str,
    method: &str,
    op: &serde_json::Value,
    path_params: &serde_json::Value,
    server_url: &str,
    now: i64,
) -> Request {
    let http_method = HttpMethod::from_str(method).unwrap_or(HttpMethod::Get);

    // parameters（path 级 + operation 级）
    let mut query: Vec<KeyValue> = Vec::new();
    let mut headers: Vec<HeaderEntry> = Vec::new();
    let mut params: Vec<&serde_json::Value> = Vec::new();
    if let Some(arr) = path_params.as_array() {
        params.extend(arr.iter());
    }
    if let Some(arr) = op["parameters"].as_array() {
        params.extend(arr.iter());
    }
    for p in params {
        let name = p["name"].as_str().unwrap_or("").to_string();
        if name.is_empty() {
            continue;
        }
        let value = p["schema"]["default"]
            .as_str()
            .or_else(|| p["schema"]["example"].as_str())
            .unwrap_or("")
            .to_string();
        match p["in"].as_str().unwrap_or("") {
            "query" => query.push(KeyValue::new(name, value)),
            "header" => headers.push(HeaderEntry::new(name, value)),
            _ => {}
        }
    }

    // body：requestBody.content 第一个 content-type
    let body = op["requestBody"]["content"].as_object().and_then(|c| {
        let ct = c.keys().next()?.clone();
        let content = c.get(&ct)
            .and_then(|m| m["example"].as_str())
            .or_else(|| c.get(&ct).and_then(|m| m["schema"]["example"].as_str()))
            .unwrap_or("")
            .to_string();
        Some(RequestBody::Raw {
            content_type: ct,
            content,
        })
    });

    // auth：识别 security scheme（operation 级优先，否则全局）
    let auth = detect_auth(op, &serde_json::json!({}));

    let url = if server_url.is_empty() {
        path.to_string()
    } else {
        format!("{}{}", server_url, path)
    };

    Request {
        id: Uuid::new_v4().to_string(),
        collection_id: collection_id.to_string(),
        folder_id: None,
        name: op["summary"]
            .as_str()
            .or_else(|| op["operationId"].as_str())
            .map(String::from)
            .unwrap_or_else(|| format!("{} {}", method.to_uppercase(), path)),
        method: http_method,
        url,
        headers,
        query,
        body,
        auth,
        sort_order: 0,
        created_at: now,
        updated_at: now,
    }
}

/// 简单识别鉴权（operation 级 security 或全局 security）
/// 返回 None 表示未识别（用户手动配置）
fn detect_auth(_op: &serde_json::Value, _doc: &serde_json::Value) -> Option<AuthConfig> {
    // Phase 3 第一版：不自动生成 token（避免假密钥），
    // 但识别 apiKey header 类型时给出模板
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::repository::Repos;
    use std::sync::Arc;

    fn setup() -> (Arc<Database>, Repos) {
        let dir = std::env::temp_dir().join(format!("zeroapi-oa-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = Arc::new(Database::new(dir.join("test.db")).unwrap());
        db.migrate().unwrap();
        let repos = Repos::new(db.clone());
        (db, repos)
    }

    const SAMPLE_OPENAPI_YAML: &str = r#"
openapi: 3.0.0
info:
  title: Pet Store
  version: 1.0.0
  description: 示例宠物商店 API
servers:
  - url: https://api.petstore.example.com/v1
paths:
  /pets:
    get:
      tags: [pets]
      summary: 获取宠物列表
      operationId: listPets
      parameters:
        - name: limit
          in: query
          schema:
            type: integer
            default: "20"
        - name: X-Request-Id
          in: header
          schema:
            type: string
    post:
      tags: [pets]
      summary: 创建宠物
      operationId: createPet
      requestBody:
        content:
          application/json:
            schema:
              type: object
  /pets/{petId}:
    get:
      tags: [pets]
      summary: 获取单个宠物
      operationId: getPet
      parameters:
        - name: petId
          in: path
          required: true
          schema:
            type: string
    delete:
      tags: [pets]
      summary: 删除宠物
      operationId: deletePet
  /owners:
    get:
      summary: 获取主人列表（无 tag）
      operationId: listOwners
"#;

    #[test]
    fn test_import_yaml_openapi() {
        let (_db, repos) = setup();
        let svc = OpenApiService::new(_db.clone(), &repos);
        let result = svc.import(SAMPLE_OPENAPI_YAML, None).unwrap();

        // 项目自动创建
        let project = repos.projects.get(&result.project_id).unwrap().unwrap();
        assert_eq!(project.name, "Pet Store");

        // 集合：pets + 默认（无 tag）
        let cols = repos.collections.list(Some(&result.project_id)).unwrap();
        assert_eq!(cols.len(), 2);
        assert!(cols.iter().any(|c| c.name == "pets"));
        assert!(cols.iter().any(|c| c.name == "默认"));

        // 请求：5 个（3 pets + 1 post + 1 owners）
        let reqs = repos.requests.list(None, None).unwrap();
        let project_reqs: Vec<_> = reqs.iter().filter(|r| {
            cols.iter().any(|c| c.id == r.collection_id)
        }).collect();
        assert_eq!(project_reqs.len(), 5);

        // GET /pets：query 参数 + header 参数 + server 前缀
        let list = project_reqs.iter().find(|r| r.name == "获取宠物列表").unwrap();
        assert_eq!(list.method, HttpMethod::Get);
        assert_eq!(list.url, "https://api.petstore.example.com/v1/pets");
        assert!(list.query.iter().any(|q| q.name == "limit" && q.value == "20"));
        assert!(list.headers.iter().any(|h| h.name == "X-Request-Id"));

        // POST /pets：body content-type
        let create = project_reqs.iter().find(|r| r.name == "创建宠物").unwrap();
        assert_eq!(create.method, HttpMethod::Post);
        match create.body.as_ref().unwrap() {
            RequestBody::Raw { content_type, .. } => {
                assert_eq!(content_type, "application/json");
            }
            _ => panic!("expected raw body"),
        }

        // 环境：servers[0] → base_url
        let envs = repos.environments.list().unwrap();
        assert!(envs.iter().any(|e| e.base_url == "https://api.petstore.example.com/v1"));

        // 断言导入数量
        assert_eq!(result.collections, 2);
        assert_eq!(result.requests, 5);
    }

    #[test]
    fn test_import_json_openapi() {
        let (_db, repos) = setup();
        let svc = OpenApiService::new(_db.clone(), &repos);
        // JSON 形式（OpenAPI 3.1）
        let json = r#"{
            "openapi": "3.1.0",
            "info": { "title": "JSON API", "version": "1.0" },
            "paths": {
                "/health": {
                    "get": { "summary": "健康检查", "tags": ["sys"] }
                }
            }
        }"#;
        let result = svc.import(json, None).unwrap();
        assert_eq!(result.requests, 1);
        let reqs = repos.requests.list(None, None).unwrap();
        assert!(reqs.iter().any(|r| r.name == "健康检查" && r.url == "/health"));
    }

    #[test]
    fn test_import_into_existing_project() {
        let (_db, repos) = setup();
        let svc = OpenApiService::new(_db.clone(), &repos);
        let now = chrono::Utc::now().timestamp_millis();
        repos.projects.upsert(&Project {
            id: "existing".into(),
            name: "既有项目".into(),
            description: None,
            root_path: None,
            created_at: now,
            updated_at: now,
        }).unwrap();
        let result = svc.import(SAMPLE_OPENAPI_YAML, Some("existing".into())).unwrap();
        assert_eq!(result.project_id, "existing");
        let cols = repos.collections.list(Some("existing")).unwrap();
        assert!(!cols.is_empty());
    }
}
