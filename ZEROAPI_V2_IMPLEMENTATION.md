# ZeroApi V2 开发改进归档文档

> 文档版本：V2.0\
> 创建时间：2026-08-17\
> 项目：ZeroApi\
> 项目定位：Local-first API Development & Network Diagnostic Workbench\
> 目标：在保持轻量、无需登录、数据本地化的前提下，把 ZeroApi 从个人 API
> 调试工具升级为可长期演进、可发布运营的专业开发者工具。

------------------------------------------------------------------------

# 1. 项目目标

## 1.1 核心定位

ZeroApi 不追求成为另一个 Postman / Apifox。

核心定位：

> **本地优先（Local-first）的 API 开发与网络诊断工作台。**

重点解决：

-   内网环境 API 调试
-   无外网环境使用
-   敏感 API 数据不上传云端
-   Git 管理 API 项目
-   OpenAPI / cURL 导入
-   请求历史与响应对比
-   DNS / TCP / TLS / HTTP 网络诊断
-   API 请求性能分析

## 1.2 核心原则

1.  不要求登录即可使用核心功能。
2.  核心 API 数据默认不离开用户设备。
3.  项目文件可被 Git 管理。
4.  Request 与 Response 解耦。
5.  HTTP Transport 与业务模型解耦。
6.  Secret 与普通 Environment Variable 分离。
7.  优先解决 API 调试刚需，再扩展高级功能。
8.  不为了"看起来完整"而盲目增加 GraphQL、AI、Mock、Runner 等功能。

------------------------------------------------------------------------

# 2. V2 总体架构

``` text
                         ZeroApi
                            │
             ┌──────────────┼──────────────┐
             │              │              │
          Request         Project        Network
          Engine          Engine         Engine
             │              │              │
       ┌─────┼─────┐    ┌───┼────┐     ┌──┼────┐
       │     │     │    │   │    │     │  │    │
      HTTP  WS   gRPC  Env Git OpenAPI DNS TCP TLS
       │
       ↓
   Response
   Execution
   History
   Diff
   Timeline
```

V2 初期只实现 HTTP，WebSocket / gRPC 预留架构接口，不立即实现。

------------------------------------------------------------------------

# 3. 当前版本与 V2 的主要问题

## P0：必须解决

### 3.1 Request 与 Response 耦合

当前 Request 包含：

-   last_response
-   status

问题：

-   Request 修改与 Response 生命周期耦合
-   无法自然实现多次执行记录
-   不利于统计性能
-   不利于 Response Diff

改造：

``` text
Request
   +
RequestExecution
   +
ResponseSnapshot
```

------------------------------------------------------------------------

### 3.2 Collection 使用 request_ids JSON

当前 Collection 维护 Request ID 列表。

问题：

-   不利于 Folder
-   不利于排序
-   不利于层级结构
-   不利于 Git / OpenAPI
-   后续维护成本高

改造为关系模型：

``` text
Project
 └── Collection
      └── Folder
           └── Request
```

------------------------------------------------------------------------

### 3.3 HTTP 请求编译逻辑与 Transport 耦合

当前 HTTP 层同时负责：

-   环境变量替换
-   URL 处理
-   Query 参数
-   Header
-   Auth
-   Body
-   HTTP 请求发送

改造：

``` text
Request
  +
Environment
  +
Secret
  ↓
RequestCompiler
  ↓
CompiledRequest
  ↓
HttpTransport
  ↓
Response
```

------------------------------------------------------------------------

### 3.4 Error 模型过于简单

当前主要依赖 reqwest Error。

V2 必须结构化：

``` text
InvalidUrl
Dns
Connection
Timeout
Tls
Proxy
Protocol
Cancelled
Unknown
```

------------------------------------------------------------------------

### 3.5 Secret 明文问题

Environment 中的：

-   JWT
-   API Key
-   Password
-   Client Secret

不能与普通变量完全等价。

引入：

``` text
Plain Variable
Secret Variable
```

Secret 默认不进入 Git 项目文件。

------------------------------------------------------------------------

### 3.6 HTTP Client Cookie 隔离

当前 HTTP Client 使用共享 Cookie Store。

V2 需要至少按 Project / Session 隔离 Cookie。

------------------------------------------------------------------------

### 3.7 Response 全量读取

当前 Response Body 直接读取为 String。

V2 需要：

-   文本预览大小限制
-   Binary 支持
-   大响应保护
-   文件保存

------------------------------------------------------------------------

### 3.8 Header 使用 HashMap

Response Header 可能存在重复 Header，例如：

``` text
Set-Cookie
```

建议统一使用：

``` rust
pub struct HeaderEntry {
    pub name: String,
    pub value: String,
}
```

------------------------------------------------------------------------

### 3.9 数据库 Migration 需要正规化

当前使用逐列检查 / ALTER TABLE 的方式。

V2 改为版本化 Migration：

``` text
001_initial.sql
002_projects.sql
003_request_execution.sql
004_secrets.sql
...
```

------------------------------------------------------------------------

# 4. V2 Domain Model

## 4.1 Project

``` rust
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub root_path: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
```

Project 是：

-   Git 管理边界
-   Environment 管理边界
-   Collection 管理边界
-   Secret 引用边界

------------------------------------------------------------------------

## 4.2 Collection

``` rust
pub struct Collection {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub created_at: i64,
    pub updated_at: i64,
}
```

------------------------------------------------------------------------

## 4.3 Folder

``` rust
pub struct Folder {
    pub id: String,
    pub collection_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub sort_order: i32,
}
```

------------------------------------------------------------------------

## 4.4 Request

``` rust
pub struct Request {
    pub id: String,
    pub collection_id: String,
    pub folder_id: Option<String>,
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<HeaderEntry>,
    pub query: Vec<KeyValue>,
    pub body: Option<RequestBody>,
    pub auth: Option<AuthConfig>,
    pub sort_order: i32,
    pub created_at: i64,
    pub updated_at: i64,
}
```

Request 不再包含：

``` text
last_response
last_status
last_execution
```

------------------------------------------------------------------------

# 5. Request Execution

新增：

``` rust
pub struct RequestExecution {
    pub id: String,
    pub request_id: String,
    pub started_at: i64,
    pub duration_ms: u64,
    pub status_code: Option<u16>,
    pub success: bool,
    pub error: Option<NetworkError>,
    pub response: Option<ResponseSnapshot>,
}
```

作用：

-   请求历史
-   性能统计
-   Replay
-   Response Diff
-   Timeline
-   错误诊断

------------------------------------------------------------------------

# 6. Response Snapshot

``` rust
pub struct ResponseSnapshot {
    pub status: u16,
    pub headers: Vec<HeaderEntry>,
    pub body: ResponseBody,
    pub size_bytes: u64,
    pub content_type: Option<String>,
    pub timing: Timing,
}
```

------------------------------------------------------------------------

# 7. Response Body

``` rust
pub enum ResponseBody {
    Text(String),
    Binary {
        path: String,
        size: u64,
    },
}
```

建议增加配置：

``` text
max_preview_size = 5MB
```

超过限制时：

-   不直接完整加载到 UI
-   支持保存文件
-   UI 显示大小与类型

------------------------------------------------------------------------

# 8. Timing

``` rust
pub struct Timing {
    pub dns_ms: Option<u64>,
    pub tcp_ms: Option<u64>,
    pub tls_ms: Option<u64>,
    pub request_ms: Option<u64>,
    pub response_ms: Option<u64>,
    pub total_ms: u64,
}
```

UI 展示：

``` text
DNS       2ms
TCP       8ms
TLS       31ms
Server    96ms
Download  5ms

Total     142ms
```

------------------------------------------------------------------------

# 9. Error Model

``` rust
pub enum NetworkErrorKind {
    InvalidUrl,
    Dns,
    Connection,
    Timeout,
    Tls,
    Proxy,
    Protocol,
    Cancelled,
    Unknown,
}
```

``` rust
pub struct NetworkError {
    pub kind: NetworkErrorKind,
    pub message: String,
    pub detail: Option<String>,
}
```

UI 必须区分：

``` text
DNS Failed
Connection Refused
Timeout
TLS Certificate Error
Proxy Error
HTTP 401
HTTP 500
```

------------------------------------------------------------------------

# 10. Environment

## 10.1 普通变量

``` text
BASE_URL
API_VERSION
TENANT_ID
```

## 10.2 Secret

``` text
JWT_TOKEN
API_KEY
CLIENT_SECRET
PASSWORD
```

类型：

``` rust
pub enum VariableKind {
    Plain,
    Secret,
}
```

------------------------------------------------------------------------

# 11. Secret 存储原则

Secret 不应该明文保存到：

``` text
Git
Project File
Export File
普通 SQLite 字段
```

推荐：

``` text
Windows Credential Manager / DPAPI
```

SQLite 只保存引用：

``` text
secret_ref
```

项目文件只保存：

``` json
{
  "name": "JWT_TOKEN",
  "type": "secret"
}
```

------------------------------------------------------------------------

# 12. RequestCompiler

新增核心模块：

``` text
src-tauri/src/transport/compiler.rs
```

职责：

``` text
Request
+
Environment
+
Secret
+
Settings
        ↓
CompiledRequest
```

结构：

``` rust
pub struct CompiledRequest {
    pub method: HttpMethod,
    pub url: Url,
    pub headers: HeaderMap,
    pub body: Option<Vec<u8>>,
}
```

处理顺序：

``` text
1. Environment substitution
2. URL parse
3. Query merge
4. Auth
5. Header merge
6. Body compile
7. Final request
```

Transport 不再负责业务变量替换。

------------------------------------------------------------------------

# 13. Transport

目录：

``` text
src-tauri/src/transport/
├── mod.rs
├── http.rs
├── websocket.rs
└── grpc.rs
```

V2 初期：

``` text
HttpTransport
```

其他只保留接口设计，不实现。

------------------------------------------------------------------------

# 14. Service 层

推荐：

``` text
Command
  ↓
Service
  ↓
Repository
  ↓
SQLite
```

不要引入过度复杂的 Java 式分层。

只保留：

``` text
Command
Service
Repository
```

------------------------------------------------------------------------

# 15. Commands 拆分

当前集中式 commands.rs 改为：

``` text
commands/
├── request.rs
├── project.rs
├── collection.rs
├── environment.rs
├── history.rs
├── import.rs
├── export.rs
├── settings.rs
└── diagnostic.rs
```

------------------------------------------------------------------------

# 16. Repository

建议：

``` text
repository/
├── request_repo.rs
├── project_repo.rs
├── collection_repo.rs
├── folder_repo.rs
├── environment_repo.rs
├── execution_repo.rs
└── history_repo.rs
```

Repository 只负责数据持久化。

不要在 Repository 里写业务逻辑。

------------------------------------------------------------------------

# 17. SQLite Schema

## projects

``` sql
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    root_path TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

## collections

``` sql
CREATE TABLE collections (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY(project_id) REFERENCES projects(id)
);
```

## folders

``` sql
CREATE TABLE folders (
    id TEXT PRIMARY KEY,
    collection_id TEXT NOT NULL,
    parent_id TEXT,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(collection_id) REFERENCES collections(id)
);
```

## requests

``` sql
CREATE TABLE requests (
    id TEXT PRIMARY KEY,
    collection_id TEXT NOT NULL,
    folder_id TEXT,
    name TEXT NOT NULL,
    method TEXT NOT NULL,
    url TEXT NOT NULL,
    headers TEXT,
    query_params TEXT,
    body TEXT,
    auth TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY(collection_id) REFERENCES collections(id),
    FOREIGN KEY(folder_id) REFERENCES folders(id)
);
```

## request_executions

``` sql
CREATE TABLE request_executions (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL,
    method TEXT NOT NULL,
    url TEXT NOT NULL,
    status_code INTEGER,
    duration_ms INTEGER,
    size_bytes INTEGER,
    content_type TEXT,
    success INTEGER NOT NULL,
    error_code TEXT,
    error_message TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(request_id) REFERENCES requests(id)
);
```

Response Body 可以根据实际实现单独保存。

------------------------------------------------------------------------

# 18. Migration

目录：

``` text
src-tauri/migrations/
├── 001_initial.sql
├── 002_projects.sql
├── 003_folders.sql
├── 004_request_execution.sql
├── 005_environment_secret.sql
└── 006_indexes.sql
```

要求：

-   Migration 可重复执行
-   明确版本
-   不依赖运行时猜测字段
-   每次 Schema 变化新增 Migration
-   不直接修改旧 Migration

------------------------------------------------------------------------

# 19. Project File Format

目标：

> Project 文件可以直接进入 Git。

推荐：

``` text
my-project/
├── zeroapi.yaml
├── collections/
│   ├── users.yaml
│   └── orders.yaml
├── environments/
│   ├── dev.yaml
│   └── test.yaml
└── scripts/
```

原则：

-   人类可读
-   Git 可 diff
-   不保存 Secret
-   不保存 Response History
-   不保存机器相关路径

------------------------------------------------------------------------

# 20. zeroapi.yaml

示例：

``` yaml
version: 1
project:
  name: Demo API
  description: Demo project

settings:
  default_environment: dev
```

------------------------------------------------------------------------

# 21. Request 文件示例

``` yaml
name: Get User
method: GET
url: "{{base_url}}/users/{{user_id}}"

query:
  - name: verbose
    value: "true"
    enabled: true

headers:
  - name: Accept
    value: application/json
    enabled: true

auth:
  type: bearer
  token: "{{JWT_TOKEN}}"
```

------------------------------------------------------------------------

# 22. Git 原则

Git 管理：

``` text
Request
Collection
Folder
Environment Template
OpenAPI
Scripts
Project Metadata
```

不进入 Git：

``` text
Secret Value
Request History
Response Cache
Local Settings
Cookie Store
```

------------------------------------------------------------------------

# 23. OpenAPI

V2.1 开始实现。

第一阶段：

``` text
OpenAPI 3.0 Import
OpenAPI 3.1 Import
```

流程：

``` text
OpenAPI
 ↓
Parser
 ↓
Project
 ↓
Collection
 ↓
Folder
 ↓
Request
```

第二阶段：

``` text
OpenAPI URL Import
```

例如：

``` text
http://localhost:8080/v3/api-docs
```

------------------------------------------------------------------------

# 24. cURL

继续保留双向转换：

``` text
ZeroApi
   ↕
 cURL
```

必须支持：

-   Import cURL
-   Export cURL
-   Header
-   Query
-   Body
-   Auth
-   Cookie

------------------------------------------------------------------------

# 25. Request Replay

History 中提供：

``` text
Replay
```

恢复：

``` text
Method
URL
Headers
Query
Body
Auth
Environment
```

Replay 后产生新的 RequestExecution。

------------------------------------------------------------------------

# 26. Response Diff

支持：

``` text
Response A
vs
Response B
```

典型场景：

``` text
DEV vs TEST
TEST vs PROD
修改前 vs 修改后
```

第一版只做 JSON Diff。

------------------------------------------------------------------------

# 27. Network Diagnostic

目录：

``` text
src-tauri/src/network/
├── dns.rs
├── tcp.rs
├── tls.rs
├── proxy.rs
└── diagnostic.rs
```

第一版：

``` text
DNS
TCP
TLS
HTTP
```

第二版：

``` text
Proxy
Route
Certificate Chain
```

------------------------------------------------------------------------

# 28. DiagnosticResult

``` rust
pub struct DiagnosticResult {
    pub target: String,
    pub dns: DnsResult,
    pub tcp: TcpResult,
    pub tls: Option<TlsResult>,
    pub http: Option<HttpResult>,
    pub total_ms: u64,
}
```

------------------------------------------------------------------------

# 29. Diagnostic UI

目标：

``` text
Network Diagnostics

Target
https://api.company.local

DNS
✓ 10.10.20.15     2ms

TCP
✓ 443             8ms

TLS
⚠ Internal CA    31ms

HTTP
✓ 200            96ms

Total
137ms
```

错误时给出：

``` text
问题：
Connection Refused

目标：
10.10.20.15:8080

建议：
1. 检查服务是否启动
2. 检查端口是否监听
3. 检查防火墙
4. 检查代理配置
```

------------------------------------------------------------------------

# 30. Request Cancellation

目标：

> 点击取消后，真正取消底层 HTTP Future。

不要只让 Tauri Command 提前返回。

推荐：

``` text
CancellationToken
        ↓
HttpTransport
        ↓
tokio::select!
```

结果：

``` text
UI Cancel
 ↓
CancellationToken.cancel()
 ↓
HTTP Future 终止
 ↓
RequestExecution = Cancelled
```

------------------------------------------------------------------------

# 31. Cookie Session

推荐：

``` text
Project
 ↓
Session
 ↓
CookieJar
 ↓
HttpClient
```

避免所有 Project 共享 Cookie。

UI 后续支持：

``` text
查看 Cookie
删除 Cookie
清空 Cookie
禁用 Cookie
```

------------------------------------------------------------------------

# 32. 前端目录

推荐逐步迁移：

``` text
src/features/
├── request/
│   ├── components/
│   ├── hooks/
│   ├── requestStore.ts
│   ├── requestService.ts
│   └── types.ts
│
├── project/
├── collection/
├── environment/
├── history/
└── diagnostics/
```

不要一次性移动全部组件。

------------------------------------------------------------------------

# 33. 前端状态原则

不要：

``` text
requestStore
 ↓
直接操作 dataStore
```

改为：

``` text
Component
 ↓
Feature Service
 ↓
Tauri API
```

Store 负责：

-   UI 状态
-   当前 Request
-   当前 Project
-   当前 Environment

Service 负责：

-   IPC
-   数据读取
-   数据保存
-   执行请求

------------------------------------------------------------------------

# 34. Tauri API

继续统一封装：

``` text
src/lib/tauri.ts
```

后续可以拆：

``` text
src/api/
├── requestApi.ts
├── projectApi.ts
├── collectionApi.ts
├── environmentApi.ts
└── diagnosticApi.ts
```

------------------------------------------------------------------------

# 35. V2 Roadmap

## Phase 1：架构重构

预计 1～2 周。

### 必须完成

-   [ ] Domain Model
-   [ ] Project
-   [ ] Folder
-   [ ] Request 重构
-   [ ] RequestExecution
-   [ ] Response 解耦
-   [ ] RequestCompiler
-   [ ] Error Model
-   [ ] Migration
-   [ ] Command 拆分
-   [ ] Service 层
-   [ ] Repository 层

### 不新增大型功能

------------------------------------------------------------------------

## Phase 2：Local-first

预计 1～2 周。

-   [ ] Project
-   [ ] Folder
-   [ ] Git-friendly Project Format
-   [ ] Import / Export V2
-   [ ] Environment
-   [ ] Secret
-   [ ] Cookie Session

------------------------------------------------------------------------

## Phase 3：API Compatibility

预计 1～2 周。

-   [ ] OpenAPI 3.0 Import
-   [ ] OpenAPI 3.1 Import
-   [ ] OpenAPI URL Import
-   [ ] cURL Import
-   [ ] cURL Export
-   [ ] Request Replay
-   [ ] Response Diff

------------------------------------------------------------------------

## Phase 4：Network Diagnostic

预计 2～3 周。

-   [ ] DNS
-   [ ] TCP
-   [ ] TLS
-   [ ] HTTP Timing
-   [ ] Proxy Diagnostic
-   [ ] Network Error
-   [ ] Diagnostic UI

------------------------------------------------------------------------

## Phase 5：质量

预计 1～2 周。

-   [ ] Unit Test
-   [ ] Integration Test
-   [ ] Migration Test
-   [ ] OpenAPI Import Test
-   [ ] cURL Import Test
-   [ ] Request Execution Test
-   [ ] Secret Security Test
-   [ ] Large Response Test
-   [ ] Cancellation Test

------------------------------------------------------------------------

## Phase 6：发布

-   [ ] Windows Installer
-   [ ] Portable Version
-   [ ] Auto Update
-   [ ] GitHub Release
-   [ ] README
-   [ ] Screenshots
-   [ ] Demo GIF
-   [ ] Issue Template
-   [ ] Feature Request Template
-   [ ] Release Notes

------------------------------------------------------------------------

# 36. 暂时明确不做

V2 初期不要做：

-   [ ] 登录
-   [ ] 云同步
-   [ ] Team
-   [ ] SaaS Backend
-   [ ] IM
-   [ ] Mock Server
-   [ ] 压测
-   [ ] GraphQL
-   [ ] WebSocket
-   [ ] gRPC
-   [ ] AI Chat
-   [ ] 在线文档
-   [ ] 移动端
-   [ ] 数据库管理

这些功能不是当前核心竞争力。

------------------------------------------------------------------------

# 37. 后续商业化方向

## Community

免费 / 开源：

-   HTTP Client
-   Project
-   Collection
-   Environment
-   cURL
-   OpenAPI
-   History
-   Local-first

## Pro

考虑收费：

-   Advanced Network Diagnostic
-   Response Diff
-   Advanced Testing
-   CLI
-   Code Generation
-   高级项目管理

## Enterprise

考虑：

-   Offline License
-   企业部署
-   Policy
-   Audit
-   SSO
-   技术支持

商业模式优先考虑：

> 一次性买断 + 企业授权

而不是强制订阅。

------------------------------------------------------------------------

# 38. 开发优先级规则

任何新功能提交前，先回答：

### Q1

是否解决 API 开发的真实问题？

### Q2

是否强化 Local-first？

### Q3

是否强化 Network Diagnostic？

### Q4

是否会增加长期维护成本？

### Q5

是否会导致用户必须登录？

### Q6

是否会引入云服务成本？

如果只是：

> "竞品有，所以我们也应该有。"

暂缓。

------------------------------------------------------------------------

# 39. Git Commit 建议

采用：

``` text
feat:
fix:
refactor:
perf:
test:
docs:
chore:
```

示例：

``` text
refactor: decouple response from request
feat: add project domain model
feat: add request execution history
refactor: introduce request compiler
feat: add secret variable support
feat: add openapi import
feat: add network diagnostic
test: add request execution integration tests
```

------------------------------------------------------------------------

# 40. 每个阶段的验收标准

## Phase 1 验收

必须做到：

``` text
旧数据
 ↓
Migration
 ↓
新 Schema
 ↓
请求发送正常
 ↓
History 正常
```

不能出现：

-   数据丢失
-   Request 无法发送
-   Import 无法使用
-   History 无法读取

------------------------------------------------------------------------

## Phase 2 验收

必须做到：

``` text
创建 Project
 ↓
创建 Collection
 ↓
创建 Request
 ↓
关闭应用
 ↓
重新打开
 ↓
数据正常
```

并且：

``` text
Export
 ↓
Git
 ↓
Clone
 ↓
Import / Open
```

结果一致。

------------------------------------------------------------------------

## Phase 3 验收

必须做到：

``` text
OpenAPI
 ↓
Import
 ↓
Collection
 ↓
Request
 ↓
Send
```

cURL：

``` text
cURL → ZeroApi → cURL
```

主要请求信息不丢失。

------------------------------------------------------------------------

## Phase 4 验收

必须做到：

``` text
正常域名
DNS ✓
TCP ✓
TLS ✓
HTTP ✓
```

异常情况：

``` text
DNS 失败
TCP 拒绝
TLS 错误
HTTP Timeout
Proxy 错误
```

都可以给出结构化错误。

------------------------------------------------------------------------

# 41. V2 最终产品结构

``` text
                         ZeroApi
                            │
             ┌──────────────┴──────────────┐
             │                             │
       API Development              Network Diagnostics
             │                             │
      ┌──────┼──────┐               ┌──────┼──────┐
      │      │      │               │      │      │
   Project Env OpenAPI             DNS    TCP    TLS
      │
 Collection
      │
 Folder
      │
 Request
      │
 RequestCompiler
      │
 HttpTransport
      │
 Response
      │
 Execution
      │
 ┌────┼─────────────┐
 │    │             │
History Replay     Diff
```

------------------------------------------------------------------------

# 42. 最终产品定位

不要把 ZeroApi 宣传成：

> "一个免费的 Postman。"

建议宣传：

> **ZeroApi --- Local-first API Development & Network Diagnostic
> Workbench**

中文：

> **ZeroApi ------ 不依赖云端的 API 开发与网络诊断工作台。**

核心卖点：

1.  无需登录
2.  数据本地保存
3.  内网环境友好
4.  Git 管理 API 项目
5.  OpenAPI / cURL
6.  网络诊断
7.  请求性能分析
8.  Windows 原生体验
9.  开源 / 可审计
10. 不依赖云端才能正常工作

------------------------------------------------------------------------

# 43. 开发执行清单

## P0

-   [ ] 新 Domain Model
-   [ ] Request / Response 解耦
-   [ ] RequestExecution
-   [ ] Project
-   [ ] Folder
-   [ ] Collection 关系化
-   [ ] RequestCompiler
-   [ ] NetworkError
-   [ ] Secret Model
-   [ ] Migration
-   [ ] Command 拆分
-   [ ] Service
-   [ ] Repository

## P1

-   [ ] Project File Format
-   [ ] Git Support
-   [ ] OpenAPI Import
-   [ ] cURL
-   [ ] Replay
-   [ ] Diff
-   [ ] Timing
-   [ ] Cookie Session

## P2

-   [ ] DNS Diagnostic
-   [ ] TCP Diagnostic
-   [ ] TLS Diagnostic
-   [ ] Proxy Diagnostic
-   [ ] Advanced Error Diagnosis

## P3

-   [ ] Script
-   [ ] Assertion
-   [ ] Runner
-   [ ] WebSocket
-   [ ] GraphQL
-   [ ] gRPC

------------------------------------------------------------------------

# 44. 开发原则总结

> **先架构，再功能。**

> **先 Local-first，再云服务。**

> **先解决内网 API 调试，再追求功能数量。**

> **先 HTTP，再考虑其他协议。**

> **先 Network Diagnostic，再 AI。**

> **先稳定单机产品，再考虑商业化。**

> **不要为了追赶 Postman 而复制 Postman。**

ZeroApi 的机会不是成为另一个 Postman。

真正值得做的是：

> **让开发者在办公室内网、医院内网、企业
> VPN、测试环境、没有公网的服务器环境中，也能拥有一个完整、轻量、可信赖的
> API 开发与网络诊断工具。**

------------------------------------------------------------------------

# 45. 当前第一开发任务

下一次开始编码时，不要直接写新功能。

第一阶段只完成：

``` text
1. 新建 domain/
2. 新建 service/
3. 新建 repository/
4. 新建 transport/
5. 新建 migration/
6. 重构 Request
7. 移除 last_response
8. 增加 RequestExecution
9. 增加 NetworkError
10. 实现 RequestCompiler
```

完成以上 10 项后，再进入 Project / Folder / Git-friendly Project
Format。

**这是 ZeroApi V2 的正式开发起点。**
