# ZeroApi V2 开发进度记录

> 更新于：2026-08-17（第四次 · Phase 3 完成）
> 依据：`ZEROAPI_V2_IMPLEMENTATION.md`
> 目标跟踪：Phase 1 → goal-c118e0e6 ✅｜Phase 2 → goal-97edf08b ✅｜Phase 3 → goal-1a50f9f1

---

## ✅ Phase 1（架构重构）— 已完成
见 git 提交 0ccf61a / a2423bf / 4714d22。

## ✅ Phase 2（Local-first）— 已完成
见 git 提交 1886876（DPAPI Secret、Git-friendly 项目文件、Cookie 管理）。

## ✅ Phase 3（API Compatibility）— 本轮完成

### 1. OpenAPI 3.0 / 3.1 Import（§23）
- `service/openapi_service.rs`：YAML/JSON 双格式解析（serde_yml → serde_json::Value）
- 映射：info.title → 项目｜servers[0].url → 环境 base_url + URL 前缀｜paths+method → Request（name=summary/operationId）｜tags[0] → Collection（无 tag → 默认）｜parameters → query/header｜requestBody.content → Body
- `import_openapi(content, project_id?)` + `import_openapi_url(url)`（reqwest 抓取）
- **3 个解析测试通过**（YAML / JSON 3.1 / 指定项目）

### 2. cURL Export（§24）
- `curl.rs::to_curl(request)`：-X / URL(query 合并) / -H / Authorization / -u / --data-raw / --data / -F
- 命令 `export_curl`；RequestPanel「复制 cURL」按钮
- **round-trip 测试通过**（导出→再解析，关键信息保留）

### 3. Request Replay（§25）
- 历史列表每项新增重放按钮：加载原请求（Method/URL/Headers/Query/Body/Auth）→ 自动发送 → 产生新 RequestExecution

### 4. Response Diff（§26）
- `components/ResponsePanel/DiffModal.tsx`：jsondiffpatch HTML 渲染（A 红 / B 绿）；非 JSON 退化逐行文本对比
- 历史列表「对比」模式：勾选两条 → DiffModal（标题含状态码）

### 5. 前端适配
- `lib/tauri.ts`：importOpenapi / importOpenapiUrl / exportCurl + OpenApiImportResult 类型
- `pages/ImportPage.tsx`：OpenAPI 面板（粘贴 YAML/JSON + URL 抓取导入）
- `store/dataStore.ts`：loadProjects

---

## ✅ 验证结果（Phase 3）

| 验证项 | 结果 |
|--------|------|
| cargo test（**15 个**：5 curl + 2 cURL导出 + 3 OpenAPI + 2 DPAPI + 2 迁移 + 1 项目 round-trip） | ✅ 全部通过 |
| cargo check / build | ✅ 通过 |
| npx tsc -b / npm run build | ✅ 通过 |

**验收达成：**
- OpenAPI 文档导入后可发送请求 ✅（导入生成项目/集合/请求，server 前缀 + 参数映射）
- cURL 导出可被 curl 直接执行 ✅（round-trip 测试）
- 历史 Replay 产生新执行记录 ✅（加载 + 自动发送）
- JSON Diff 正确展示差异 ✅（jsondiffpatch 渲染 + 文本退化）

---

## ⏭️ 下一步（文档 roadmap）

### Phase 4：Network Diagnostic
- [ ] DNS / TCP / TLS 分段 Timing（Timing 结构已就位，补探测实现）
- [ ] Network Diagnostic 命令 + UI（文档 §27-29：DNS ✓ / TCP ✓ / TLS ⚠ / HTTP ✓ 展示）
- [ ] Proxy Diagnostic

### Phase 5：质量
- [ ] 单元/集成测试扩充 · Migration Test · Secret Security Test · 大响应 Test · 取消 Test

### Phase 6：发布
- [ ] Windows Installer / Portable / Auto Update / Release

---

## 📌 关键设计备忘（Phase 3）

1. **OpenAPI**：tags 分组建集合；servers[0] 生成环境；parameters 的 default/example 填充 query 值；detect_auth 第一版返回 None（避免假 token，用户手动配）
2. **cURL 导出**：query + apiKey(in query) 合并进 URL；单引号转义 `'\\''`；测试验证 parse(to_curl(x)) 保留关键信息
3. **Replay**：getRequest(request_id) → dispatch load-request → setTimeout 50ms 后 send()
4. **Diff**：jsondiffpatch（自带 TS 类型，无需 @types）；`'jsondiffpatch/formatters/html'` 通过 exports 映射到 lib/formatters/html
5. **jsondiffpatch**：`htmlFormatter.format(delta, a)` 返回 `string | undefined`，需 `?? ''`

---

## 📌 会话备忘

- 用户 git 配置：name=jshmztl，email=jshmztl@gmail.com；git 代理 127.0.0.1:7897
- Rust 1.97.1 / npm 依赖就绪（新增 jsondiffpatch）
- Phase 3 改动：service/openapi_service.rs（新）、curl.rs（to_curl）、service/mod.rs、commands/{import,request}.rs、lib.rs、src/lib/tauri.ts、src/pages/ImportPage.tsx、src/components/ResponsePanel/DiffModal.tsx（新）、src/components/Layout/Sidebar.tsx、src/components/RequestPanel/RequestPanel.tsx、src/store/dataStore.ts、package.json
