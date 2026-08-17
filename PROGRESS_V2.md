# ZeroApi V2 开发进度记录

> 更新于：2026-08-17（第三次 · Phase 2 完成）
> 依据：`ZEROAPI_V2_IMPLEMENTATION.md`
> 目标跟踪：Phase 1 → goal-c118e0e6（已 complete）；Phase 2 → goal-97edf08b

---

## ✅ Phase 1（架构重构）— 已完成

见 git 提交 0ccf61a / a2423bf / 4714d22。全部编译、测试、运行验证通过。

## ✅ Phase 2（Local-first）— 本轮完成

### 1. Git-friendly Project File Format（文档 §19-22）
- `domain/project_file.rs`：zeroapi.yaml（version/project/settings）+ collections/*.yaml + environments/*.yaml 模型
- `service/project_file.rs`：`export_project(project_id, dir)` / `import_project(dir)`（新项目导入，folder 按名重建）
- Secret 只导出名称与类型（value 清空），普通变量完整导出
- 命令：`export_project` / `import_project`
- **round-trip 测试通过**（导出→导入数据一致；断言 Secret 明文绝不出现于项目文件）

### 2. Secret 安全存储（文档 §11）
- `security.rs`：Windows DPAPI（CryptProtectData/CryptUnprotectData，CURRENT_USER scope），非 Windows base64 兜底
- `environment_repo`：secret 变量值 DPAPI 加密 → env_secret_refs 表；vars JSON 只存引用（secret_ref）；读取时解密注入
- 导出 JSON / 项目文件均剔除 Secret 明文（Phase 1 的 export_service 已保证）
- **DPAPI 加密测试通过**（roundtrip + 每次加密产生不同密文）

### 3. Import / Export V2
- 项目级目录导入导出（上述）+ 现有 JSON 全量导入导出保留

### 4. Cookie Session 管理（文档 §31）
- `transport/session.rs`：`cookies_for(project_id, url)`（按域名查询）+ `clear(project_id)`（已有）+ `count()`
- 命令：`get_session_cookies` / `clear_session_cookies`
- 前端设置页：项目下拉 + 导出/导入目录 + Cookie 查看/清空 UI

### 5. 前端适配
- `lib/tauri.ts`：新增 exportProject / importProject / getSessionCookies / clearSessionCookies
- `pages/SettingsPage.tsx`：新增「项目文件」Section（项目选择、导出/导入目录、Secret 提示）与「Cookie 会话」Section（URL 查询、复制、清空）

---

## ✅ 验证结果（Phase 2）

| 验证项 | 结果 |
|--------|------|
| cargo check | ✅ 通过 |
| cargo test（10 个：5 curl + 2 迁移 + 2 DPAPI + 1 项目 round-trip） | ✅ 全部通过 |
| npx tsc -b | ✅ 通过 |
| npm run build（vite） | ✅ 通过 |
| cargo build（debug 完整链接） | ✅ 通过 |

**验收达成：**
- 项目导出目录可被 Git 管理 ✅（零api.yaml + collections/ + environments/，人类可读）
- 重新导入数据一致 ✅（round-trip 测试）
- Secret 明文不出现在项目文件与导出 JSON ✅（测试断言）
- Cookie 按项目隔离可管理 ✅（查询/清空命令 + UI）

---

## ⏭️ 下一步（文档 roadmap）

### Phase 3：API Compatibility
- [ ] OpenAPI 3.0 / 3.1 Import（Parser → Project → Collection → Folder → Request）
- [ ] OpenAPI URL Import
- [ ] cURL Export（当前只有 Import）
- [ ] Request Replay（历史 → 恢复请求再执行，产生新 execution）
- [ ] Response Diff（JSON Diff，DEV vs TEST 等）

### Phase 4：Network Diagnostic
- [ ] DNS / TCP / TLS / HTTP Timing 分段
- [ ] Proxy Diagnostic
- [ ] Diagnostic UI（文档 §29 的展示形式）

### Phase 5/6：质量与发布
- [ ] 单元/集成测试扩充 · Windows Installer · Release

---

## 📌 关键设计备忘（Phase 2）

1. **Secret 链路**：UI 输入明文 → repository 加密（DPAPI）→ env_secret_refs 表密文 → 读取解密注入 → compiler 使用；导出时值清空
2. **FK 顺序**：environment upsert 先落环境行再写 secret refs（避免外键约束失败）
3. **项目文件**：文件名 slug 化（中文名 → untitled.yaml + 序号去重）；导入生成全新 id（不覆盖现有数据）
4. **Cookie**：Jar::cookies(url) 需 `use reqwest::cookie::CookieStore` trait
5. **windows-sys 0.59**：DATA_BLOB 已更名 CRYPT_INTEGER_BLOB；LocalFree 在 Win32::Foundation

---

## 📌 会话备忘

- 用户 git 配置：name=jshmztl，email=jshmztl@gmail.com；git 代理 127.0.0.1:7897
- Rust 1.97.1 / npm 依赖已就绪
- Phase 2 改动文件：src-tauri/src/security.rs（新）、domain/{environment,project_file,mod}.rs、repository/environment_repo.rs、service/{mod,project_file}.rs、transport/session.rs、commands/{project,settings}.rs、lib.rs、Cargo.toml、src/lib/tauri.ts、src/pages/SettingsPage.tsx
