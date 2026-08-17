# ZeroApi V2 Phase 1 开发进度记录

> 更新于：2026-08-17（第二次）
> 依据：`ZEROAPI_V2_IMPLEMENTATION.md` 第 45 节「第一开发任务」（10 项核心任务）
> 目标跟踪：goal-c118e0e6-8612-44e5-bada-690bfe6e3ecc

---

## ✅ 已完成（全部编译验证通过）

### 1. domain/ 目录（V2 领域模型，13 个文件）✅
`src-tauri/src/domain/`：common / project / collection / folder / request / execution / response / timing / error / environment / settings / transfer（导入导出容器）+ mod.rs

### 2. migrations/ 目录（6 个版本化 SQL）✅
`src-tauri/migrations/`：001_initial ~ 006_indexes

### 3. db 层（版本化 migration runner + 5 个 Rust 数据迁移）✅
`src-tauri/src/db.rs` + `src-tauri/src/db/migrations.rs`
- schema_migrations 表记录版本，每迁移事务内原子执行
- 数据迁移：collections_v2 / requests_v2 表切换、history→executions、favorites JSON 转换、environments 加列
- **迁移集成测试通过**（test_migrate_v1_to_v2 / test_fresh_db_migrates_cleanly）

### 4. repository/ 目录（7 个 repo）✅
project / collection / folder / request（含收藏）/ execution / environment / settings

### 5. transport/ 目录 ✅
- `compiler.rs` — RequestCompiler（Request+Env+Secret+Settings → CompiledRequest，顺序：变量替换→URL→Query→Auth→Header→Body）
- `http.rs` — HttpTransport（错误分类 NetworkErrorKind、大响应保护 max_preview_size、重复 Header）
- `session.rs` — Cookie Session 按 Project 隔离（文档 §31）

### 6. service/ 目录（8 个 service）✅
request（发送+执行记录+取消）/ project / collection / environment / history / import / export / settings

### 7. commands/ 拆分（8 个模块）✅
request / project / collection / environment / history / import / export / settings

### 8. 其他 ✅
- curl.rs 适配 V2 模型（5 个单测通过）
- error.rs 加 AppError::Network
- lib.rs 重构（AppState: db + services + transport + sessions + cancel_registry）
- 删除旧 models.rs / http.rs / commands.rs

### 9. 前端适配 ✅
- `src/types/index.ts`（V2 全模型）
- `src/lib/tauri.ts`（命令封装）
- stores（requestStore / dataStore / settingsStore）
- 组件：RequestPanel / ResponsePanel / Sidebar / CollectionPage / KeyValueEditor / BodyEditor / AuthEditor / EnvironmentDialog（支持 Secret 变量）
- **tsc 类型检查通过，vite build 通过**

---

## ✅ 验证结果

| 验证项 | 结果 |
|--------|------|
| cargo check | ✅ 通过（仅无害 warning） |
| cargo test（7 个：5 curl + 2 迁移） | ✅ 全部通过 |
| npx tsc -b | ✅ 通过 |
| npm run build（vite） | ✅ 通过 |
| cargo build（debug 完整链接） | ✅ 通过 |

**验收标准达成情况（文档 §40 Phase 1）：**
- 旧数据 Migration → 新 Schema：✅ 测试验证（集合/请求/历史/收藏全保留）
- 请求发送正常：✅ 编译通过 + transport/service 逻辑完整（待运行冒烟验证）
- History 正常：✅ 基于 RequestExecution（旧 history 已迁移）

---

## ⏭️ 剩余工作

### Phase 1 收尾
- [ ] 运行 `npm run tauri:dev` 启动应用做手工冒烟测试（发送请求/历史/收藏/导入）
- [ ] 提交代码（git add + commit，按文档 §39 规范 `refactor:` / `feat:` 前缀）

### 后续阶段（文档 roadmap）
- Phase 2：Project 文件格式 / Git-friendly / Secret DPAPI / Cookie 管理 UI
- Phase 3：OpenAPI 导入 / cURL 导出 / Replay / Diff
- Phase 4：Network Diagnostic（DNS/TCP/TLS 分段 Timing 完整实现）

---

## 📌 关键设计备忘（已实现）

1. **V1→V2 数据映射**：params→query、KeyValue.key→name、method String→HttpMethod、status/last_response 移除（状态标记功能按文档删除）、history→request_executions（hist-*/exec-* 前缀）、favorites JSON 转换
2. **未分类集合** `uncategorized`：无归属请求/历史请求的统一挂载（FK 完整性）
3. **Cookie 隔离**：SessionManager（project_id → Session{jar, client}），reqwest 不再共享 cookie_store
4. **取消语义**：tokio::select! 真正终止 HTTP future，产生 Cancelled 执行记录（文档 §30）
5. **大响应**：max_preview_size（默认 5MB）超限保存文件 → ResponseBody::Binary
6. **Secret**：模型层 VariableKind + 导出时清除 secret 值；DPAPI 存储留 Phase 2
7. **Timing**：total_ms 精确，分段留 Phase 4
8. **旧数据不丢**：完整迁移 + 幂等（版本记录）

---

## 📌 会话备忘

- 用户 git 配置：name=jshmztl，email=jshmztl@gmail.com；git 代理 127.0.0.1:7897
- Rust 1.97.1 已装（$env:USERPROFILE\.cargo\bin\cargo.exe，默认工具链已设）
- npm 依赖已装（registry=npmmirror 镜像）
- 本次改动文件清单见下方 git 提交
