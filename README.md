# ZeroApi

> **零登录 · 零云端 · 零 CORS** 的轻量级 API 调试工具
>
> 基于 Tauri 2 + Rust 构建的 Windows 桌面应用 · 包小 (~8MB) · 启动快 · 隐私安全

[![Version](https://img.shields.io/badge/version-1.2.0-1FA3D2)](https://github.com/jshmztl/zeroApi/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%2011-0078D4)](https://github.com/jshmztl/zeroApi)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.x-FFC131)](https://tauri.app)

---

## ✨ 特性

| | |
|---|---|
| 🪶 **轻量** | 安装包 ~8MB,秒开,内存占用低 |
| 🔒 **隐私** | 全本地 SQLite,数据永不离开你的电脑;Secret 用 Windows DPAPI 加密落库,明文不落库 |
| 🚫 **零 CORS** | Rust 原生 HTTP 请求,绕过浏览器同源策略 |
| 🚀 **零登录** | 下载即用,不收集任何身份信息 |
| 📚 **项目 / 文件夹 / 集合** | 三级组织模型,项目文件 Git-friendly,可整体导入导出 |
| 🌐 **环境变量** | `{{var}}` 占位符 + 多环境切换,环境变量与 Secret 分开管理 |
| 🕷️ **网络诊断** | DNS / TCP / TLS / HTTP 四级时序分段(Timing),定位慢在哪 |
| 🔄 **重放 & Diff** | 历史请求一键重放,两次执行响应对比 |
| 📥 **导入 / 导出** | OpenAPI 3.0/3.1(文本+URL) / cURL 双向 / JSON 备份 / 项目文件 |
| ⚡ **请求取消** | 传输中可随时中止 |
| 🍪 **Cookie 会话** | 按项目隔离 Session,自动保存 Cookie |
| 🎨 **仪器控制台 UI** | Ink & Signal 主题:石墨画布 + 蓝图网格 + 磷光青信号色,自托管可变字体,离线可用 |
| 🔄 **自动更新** | minisign 签名校验,GitHub Releases 一键升级 |

---

## 📸 截图

> 待补 · 首发 Release 后将在此附上主工作区 / 诊断页 / 集合页截图

---

## 🚀 快速开始

### 方式一 · 下载安装包 (推荐)

前往 [Releases](https://github.com/jshmztl/zeroApi/releases) 下载最新版:

| 文件 | 说明 |
|------|------|
| `ZeroApi_x.x.x_x64-setup.exe` | NSIS 安装版 (推荐) |
| `ZeroApi_x.x.x_x64-setup.msi` | MSI 安装版 |
| `ZeroApi_x.x.x_x64-portable.exe` | 便携版 (解压即用,零安装) |

> **要求**: Windows 11 (x64 / ARM64) · 系统自带 WebView2 Runtime

### 方式二 · 从源码运行

需要环境:
- [Node.js](https://nodejs.org) >= 18
- [Rust](https://rustup.rs) >= 1.77
- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Windows 11 + [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)

```bash
# 克隆
git clone https://github.com/jshmztl/zeroApi.git
cd zeroApi

# 安装依赖
npm install

# 启动开发模式 (HMR 热重载)
npm run tauri:dev

# 构建发布版
npm run tauri:build
# 产物在 src-tauri/target/release/bundle/
```

---

## ⌨️ 快捷键

| 快捷键 | 操作 |
|--------|------|
| `Ctrl + Enter` | 发送请求 |
| `Ctrl + S` | 收藏当前请求 |
| `Ctrl + L` | 清空当前请求 |

---

## 📦 功能清单

### HTTP 请求

- [x] 七种方法 (`GET / POST / PUT / PATCH / DELETE / HEAD / OPTIONS`)
- [x] Query 参数 / Header / Body (`form-data` / `x-www-form-urlencoded` / `raw JSON/XML/Text`)
- [x] Auth (`None / Bearer / Basic / API Key`)
- [x] 请求取消 (传输中随时中止)
- [x] Cookie 按项目 Session 隔离

### 响应

- [x] 状态码 / 耗时 / 大小 / Body / Headers / Cookies
- [x] JSON 自动美化 + Shiki 语法高亮 (Pretty / Raw 切换)
- [x] Response 与 Request 解耦执行快照

### 组织管理

- [x] 项目 (Project) / 文件夹 (Folder) / 集合 (Collection) 三级模型
- [x] 历史记录 (自动保存 + 裁剪) / 收藏夹
- [x] 请求重放 (Replay) / 响应对比 (Diff)
- [x] 环境变量 `{{var}}` + 多环境切换;Secret 单独管理 (DPAPI 加密,不进项目文件)
- [x] 项目文件 Git-friendly 格式,项目整体导入 / 导出

### 导入 / 导出

- [x] cURL 导入 / 导出
- [x] OpenAPI 3.0 / 3.1 导入 (文本 + URL)
- [x] JSON 完整备份

### 网络诊断

- [x] DNS 解析 / TCP 连接 / TLS 握手 / HTTP 响应,逐级 Timing 分段
- [x] 诊断页可视化 (内置网络诊断工作台)

### 其它

- [x] 设置 (主题 / 超时 / 代理 / SSL / 自动保存 / 请求历史)
- [x] **应用内自动更新** (minisign 签名校验 + GitHub Releases)

### 明确不做 (设计取舍)

- ❌ 用户系统 / 登录
- ❌ 云端同步 / 团队协作
- ❌ Mock Server / 性能压测
- ❌ WebSocket / GraphQL / gRPC
- ❌ Pre-request 脚本

---

## 🛠️ 技术栈

| 层 | 选型 |
|----|------|
| **桌面框架** | Tauri 2 (Rust + WebView2) + 5 插件 |
| **前端** | React 18 + TypeScript + Vite |
| **样式** | Tailwind CSS + Ink & Signal 仪器控制台主题 (离线自托管字体: Saira / Chivo / JetBrains Mono) |
| **UI 组件** | 自研 shadcn 风格 (无 Radix 依赖) |
| **HTTP 客户端** | reqwest + rustls (纯 Rust,支持 `.pem`/`.der` 证书) |
| **数据库** | SQLite (rusqlite, bundled, 版本化 migration) |
| **安全** | Windows DPAPI (Secret 加密落库) |
| **代码编辑器** | Monaco Editor |
| **语法高亮** | Shiki |
| **状态管理** | Zustand |
| **路由** | React Router |
| **网络诊断** | tokio + rustls + webpki-roots,分阶段 Timing |
| **自动更新** | tauri-plugin-updater + minisign + GitHub Releases |

---

## 📂 项目结构

```
zeroapi/
├── src/                     # React 前端
│   ├── components/
│   │   ├── ui/              # 基础组件 (Button/Input/Tabs/Toast…)
│   │   ├── Layout/          # Topbar + Sidebar
│   │   ├── RequestPanel/    # 请求配置区
│   │   ├── ResponsePanel/   # 响应展示区
│   │   ├── KeyValueEditor/  # Params/Headers 键值编辑器
│   │   └── CodeEditor/      # Monaco 封装
│   ├── pages/               # HomePage / SettingsPage / ImportPage / CollectionPage / DiagnosticsPage
│   ├── store/               # dataStore / requestStore / settingsStore (只存 UI 状态)
│   ├── lib/                 # tauri 封装 / 格式化 / 高亮
│   ├── types/               # 与 Rust 对齐的类型定义
│   └── styles/globals.css   # 全局样式 + 主题变量
├── src-tauri/               # Rust 后端 (分层)
│   ├── src/
│   │   ├── main.rs / lib.rs # 入口 + Tauri Builder
│   │   ├── commands/        # IPC 命令 (按域拆分: collection/diagnostic/environment/…)
│   │   ├── domain/          # 领域模型 (request/response/execution/project/…)
│   │   ├── repository/      # 仅 SQL 访问层
│   │   ├── service/         # 业务编排 (import/export/openapi/project_file/…)
│   │   ├── transport/       # RequestCompiler + HttpTransport + Session
│   │   ├── network/         # dns / tcp / tls 的诊断与时序
│   │   ├── security.rs      # DPAPI 加密
│   │   ├── db/              # SQLite + 版本化 migration
│   │   ├── curl.rs          # cURL 解析器
│   │   └── error.rs         # 统一错误类型
│   ├── capabilities/        # Tauri 权限
│   ├── icons/               # 多尺寸图标
│   ├── tauri.conf.json      # bundle 目标: msi / nsis / portable,含 updater.pubkey
│   └── Cargo.toml
├── scripts/
│   └── gen_icons.py         # 重新生成图标
├── .github/workflows/       # CI / 发布
│   ├── ci.yml               # 基础校验
│   └── build.yml            # Windows x64/ARM64 打包 + 自动 Release
├── SECURITY.md              # 密钥管理与安全说明
└── README.md
```

---

## 🏗️ 开发指南

### 日常开发流程

```bash
# 1. 启动开发 (HMR 热重载)
npm run tauri:dev

# 2. 改 Rust 代码 → 自动重启
# 3. 改前端代码 → 自动 HMR
```

### 添加新的 Tauri 命令

分层目录下都有对应文件,按域放置:

1. 在 `src-tauri/src/commands/<domain>.rs` 加 `#[tauri::command]`
2. 在 `src-tauri/src/commands/mod.rs` 与 `lib.rs` 的 `invoke_handler!` 注册
3. 在 `src/lib/tauri.ts` 加封装
4. 前端调用

### 数据库迁移

修改 `src-tauri/src/db/migrations.rs` 的 migration 列表,新增表 / 字段 / 版本。首次启动自动迁移。

### 修改图标

```bash
# 编辑 scripts/gen_icons.py 后:
python3 scripts/gen_icons.py
```

---

## 📦 打包发布

### 本地打包 (Windows)

```bash
npm run tauri:build
# 产物路径:
#   src-tauri/target/release/bundle/msi/*.msi
#   src-tauri/target/release/bundle/nsis/*-setup.exe   (NSIS 安装版)
#   src-tauri/target/release/bundle/nsis/*-portable.exe (便携版)
```

> 本地打包若想生成带签名的 `latest.json`,需要配置签名密钥(见下)。

### 自动发布 (推荐)

打 tag 即触发 GitHub Actions,`build.yml` 在 **Windows x64 (`windows-latest`)** 与 **ARM64 (`windows-11-arm`)** 上并行构建,自动产出 **NSIS / MSI / 便携版** 并创建 Release(**自动发布**,无需手动 Publish):

```bash
git tag v1.2.1
git push origin v1.2.1
```

### ⚠️ 发布前必配:自动更新签名

应用内更新使用 **minisign** 签名校验,`latest.json` 只有在打包时用私钥签名才会生成。首次发布前需在 GitHub **Settings → Secrets and variables → Actions** 配置**两个 Repository secrets**:

| Secret 名 | 值 |
|-----------|-----|
| `TAURI_SIGNING_PRIVATE_KEY` | minisign 私钥文件全文 (`zerokey.key`) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 该私钥的密码 |

私钥与密码请妥善备份、**严禁提交到仓库**(`src-tauri/.signing/` 已在 `.gitignore` 忽略)。公钥已内置在 `tauri.conf.json` 的 `updater.pubkey`。详见 [SECURITY.md](SECURITY.md)。

> 若这两个 Secret 缺失,构建会因无法签名而失败,或生成不了 `latest.json`,应用内更新将静默失效。

### 更新服务器配置

应用内自动更新通过 `tauri.conf.json` 的 `plugins.updater.endpoints` 拉取,每次 Release 由 Tauri Action 自动生成 `latest.json`。

---

## 🐛 常见问题

### 启动时提示"未知发布者"?

应用**二进制尚未做 EV 代码签名**,属于预期行为,点击「仍要运行」即可。这不影响功能与自动更新(minisign 只做更新包校验)。未来计划引入 EV 代码签名。

### WebView2 缺失?

Win11 自带 WebView2。如使用 Win10 或精简版系统,需手动安装:
<https://developer.microsoft.com/microsoft-edge/webview2/>

### 数据存在哪里?Secret 安全吗?

```
%APPDATA%\com.zeroapi.app\zeroapi.db
```

数据库存本地;环境变量中的 **Secret 用 Windows DPAPI 加密后写入**,明文不落库、不导出到项目文件/JSON,避免敏感信息随备份外泄。清空全部数据 = 删除该文件。

### cURL 解析失败?

- 确认命令以 `curl ` 开头
- 复杂 shell 变量展开 (如 `$VAR`) 不支持,先手动替换
- 报告 issue 时附上原始命令

### 跨域 (CORS) 真的没限制吗?

✅ **是的**。ZeroApi 不通过浏览器发请求,而是 Rust reqwest 直连目标服务器,所以**没有任何 CORS 限制**。这是它相对 Postman / Apifox 浏览器版的核心理由之一。

---

## 🗺️ Roadmap

- [x] **V1.0** (2026-07) — 核心请求 / cURL 导入 / 收藏集合 / 环境变量 / 自动更新
- [x] **V2.0** (2026-08) — 分层架构重构 / 项目·文件夹·集合 / Secret 加密 / Cookie 会话 / 网络诊断+Timing / OpenAPI·cURL 导入导出 / 重放·Diff / 请求取消
- [ ] **候选(未排期)** — Proxy 诊断 / 路由·证书链 / Code Generation / CLI / 高级测试脚本
- [ ] **明确不做** — WebSocket / GraphQL / Spec-Gen / Mock Server / 压测 / 云同步 / 团队协作

> 想提新功能?请先阅读 [Feature Request 模板](.github/ISSUE_TEMPLATE/feature_request.md) 中的定位契合度说明。

---

## 🤝 贡献

欢迎 PR / Issue(请使用 [Bug](.github/ISSUE_TEMPLATE/bug_report.md) / [Feature](.github/ISSUE_TEMPLATE/feature_request.md) 模板)!网络相关 Bug 建议附上「网络诊断」页结果。

```bash
git clone https://github.com/jshmztl/zeroApi.git
cd zeroApi
npm install
npm run tauri:dev
```

---

## 📄 License

[MIT](LICENSE) © 2026 ZeroApi

---

## 🙏 致谢

- [Tauri](https://tauri.app) — 革命性的桌面应用框架
- [shadcn/ui](https://ui.shadcn.com) — 设计灵感
- [Vite](https://vitejs.dev) — 极速前端构建
- [reqwest](https://github.com/seanmonstar/reqwest) — Rust HTTP 客户端
- [Postman](https://postman.com) — 不是致敬,是对照参考 😂

---

<p align="center">
  <sub>用 💚 制作 · 愿你少装客户端,多调接口</sub>
</p>