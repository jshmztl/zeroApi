# ZeroApi

> **零登录 · 零云端 · 零 CORS** 的轻量级 API 调试工具
>
> 基于 Tauri 2 + Rust 构建的 Windows 桌面应用 · 包小 (~8MB) · 启动快 · 隐私安全

[![Version](https://img.shields.io/badge/version-1.0.0-7FC8A9)](https://github.com/jshmztl/zeroApi/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%2011-0078D4)](https://github.com/jshmztl/zeroApi)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.x-FFC131)](https://tauri.app)

---

## ✨ 特性

| | |
|---|---|
| 🪶 **轻量** | 安装包 ~8MB,秒开,内存占用低 |
| 🔒 **隐私** | 全本地 SQLite,数据永不离开你的电脑 |
| 🚫 **零 CORS** | Rust 原生 HTTP 请求,绕过浏览器同源策略 |
| 🚀 **零登录** | 下载即用,不收集任何身份信息 |
| 📋 **cURL 导入** | 一键解析 DevTools / 终端复制的 cURL |
| 🌐 **环境变量** | `{{var}}` 占位符,多环境切换 |
| ⭐ **收藏 & 集合** | 常用请求 / 分组管理 |
| 🎨 **清新 UI** | 薄荷绿主色,留白克制,专业不喧宾夺主 |
| 🔄 **自动更新** | 内置 Tauri Updater,GitHub Releases 一键升级 |

---

## 📸 截图

> 待补 · 将在首次 Release 时附上

主工作区:

```
┌─────────────────────────────────────────────────────────────┐
│ [GET ▼] [https://api.example.com/users               ] [🚀] │
├─────────────────────────────────────────────────────────────┤
│  Params · Headers · Body · Auth                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  ✓  page    1                                        │   │
│  │  ✓  limit   20                                       │   │
│  └─────────────────────────────────────────────────────┘   │
│                              ┌──────────────────────────┐  │
│                              │ 200 OK · 142ms · 1.2KB  │  │
│                              │ {                        │  │
│                              │   "code": 0,             │  │
│                              │   "data": [...]          │  │
│                              │ }                        │  │
│                              └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 🚀 快速开始

### 方式一 · 下载安装包 (推荐)

前往 [Releases](https://github.com/jshmztl/zeroApi/releases) 下载:

| 文件 | 说明 |
|------|------|
| `ZeroApi_x.x.x_x64-setup.exe` | NSIS 安装版 (推荐) |
| `ZeroApi_x.x.x_x64-setup.msi` | MSI 安装版 |
| `ZeroApi_x.x.x_x64.exe` | 便携版 (解压即用) |

> **要求**: Windows 11 (x64 / ARM64) · 系统自带 WebView2

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

# 启动开发模式
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

## 📦 功能清单 (V1.0)

- [x] HTTP 七种方法 (`GET / POST / PUT / PATCH / DELETE / HEAD / OPTIONS`)
- [x] Query 参数 / Header / Body (`form-data` / `x-www-form-urlencoded` / `raw JSON/XML/Text`)
- [x] Auth (`None / Bearer / Basic / API Key`)
- [x] 响应展示:状态码 / 耗时 / 大小 / Body / Headers / Cookies
- [x] JSON 自动美化 + Shiki 语法高亮
- [x] cURL 命令导入解析
- [x] 历史记录 (最近 100 条)
- [x] 收藏夹
- [x] 集合 (Collection) 管理
- [x] 环境变量 `{{var}}` 替换 + 多环境切换
- [x] 导入 / 导出 JSON 备份
- [x] 设置 (主题 / 超时 / 代理 / SSL / 自动更新)
- [x] **应用内自动更新** (Tauri Updater + GitHub Releases)

### V1.0 不做

- ❌ 用户系统 / 登录
- ❌ 云端同步 / 团队协作
- ❌ Mock Server / 性能压测
- ❌ WebSocket / GraphQL
- ❌ Pre-request 脚本
- ❌ 代码签名 (V1 接受"未知发布者"提示)

---

## 🛠️ 技术栈

| 层 | 选型 |
|----|------|
| **桌面框架** | Tauri 2 (Rust + WebView2) |
| **前端** | React 18 + TypeScript + Vite |
| **样式** | Tailwind CSS |
| **UI 组件** | 自研 shadcn 风格 (无 Radix 依赖) |
| **HTTP 客户端** | reqwest + rustls |
| **数据库** | SQLite (rusqlite, bundled) |
| **代码编辑器** | Monaco Editor |
| **语法高亮** | Shiki |
| **状态管理** | Zustand |
| **路由** | React Router |
| **自动更新** | tauri-plugin-updater + GitHub Releases |

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
│   ├── pages/               # HomePage / SettingsPage / ImportPage / CollectionPage
│   ├── store/               # Zustand stores
│   ├── lib/                 # tauri 封装 / 格式化 / 高亮
│   ├── types/               # 与 Rust 对齐的类型定义
│   └── styles/globals.css   # 全局样式 + 主题变量
├── src-tauri/               # Rust 后端
│   ├── src/
│   │   ├── main.rs          # 入口
│   │   ├── lib.rs           # Tauri Builder
│   │   ├── models.rs        # 数据模型 (与前端共享)
│   │   ├── db.rs            # SQLite 封装
│   │   ├── http.rs          # reqwest 客户端
│   │   ├── curl.rs          # cURL 解析器
│   │   ├── commands.rs      # Tauri IPC 命令
│   │   └── error.rs         # 统一错误类型
│   ├── capabilities/        # Tauri 权限
│   ├── icons/               # 多尺寸图标
│   ├── tauri.conf.json
│   └── Cargo.toml
├── scripts/
│   └── gen_icons.py         # 重新生成图标
├── .github/workflows/       # CI / 发布
│   ├── ci.yml               # 基础校验
│   └── build.yml            # Windows 打包 + Release
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

1. 在 `src-tauri/src/commands.rs` 加 `#[tauri::command]`
2. 在 `src-tauri/src/lib.rs` 的 `invoke_handler!` 注册
3. 在 `src/lib/tauri.ts` 加封装
4. 前端调用

### 数据库迁移

修改 `src-tauri/src/db.rs` 的 `migrate()` 方法,加新表 / 新字段。首次启动会自动建表。

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
#   src-tauri/target/release/bundle/nsis/*.exe
```

### 自动发布 (推荐)

打 tag 即可触发 GitHub Actions:

```bash
git tag v1.0.1
git push origin v1.0.1
```

`.github/workflows/build.yml` 会在 windows-latest (x64) + windows-11-arm (ARM64) 上并行构建,并自动创建 GitHub Release(草稿),你只需在网页上点"Publish"。

### 更新服务器配置

应用内自动更新通过 `tauri.conf.json` 的 `plugins.updater.endpoints` 拉取:

```json
"updater": {
  "endpoints": [
    "https://github.com/jshmztl/zeroApi/releases/latest/download/latest.json"
  ]
}
```

每次 Release 会自动生成 `latest.json`(Tauri Action 负责)。

---

## 🐛 常见问题

### 启动时提示"未知发布者"?

V1.0 暂未签名,这是预期行为。点击「仍要运行」即可。
未来 V1.1 计划添加 EV 代码签名。

### WebView2 缺失?

Win11 自带 WebView2。如使用 Win10 或精简版系统,需手动安装:
<https://developer.microsoft.com/microsoft-edge/webview2/>

### 数据存在哪里?

```
%APPDATA%\com.zeroapi.app\zeroapi.db
```

清空所有数据 = 删除该文件(或在设置页一键清空)。

### cURL 解析失败?

- 确认命令以 `curl ` 开头
- 复杂 shell 变量展开 (如 `$VAR`) 不支持,先手动替换
- 报告 issue 时附上原始命令

### 跨域 (CORS) 真的没限制吗?

✅ **是的**。ZeroApi 不通过浏览器发请求,而是 Rust reqwest 直连目标服务器,所以**没有任何 CORS 限制**。这是它相对 Postman / Apifox 浏览器版的核心理由之一。

---

## 🗺️ Roadmap

- [x] **V1.0** (2026-07) — 核心功能 + cURL + 自动更新
- [ ] **V1.1** — 暗色模式 / 响应 Preview / Header 模板 / 快捷键大全
- [ ] **V2.0** — 断言 / 脚本 / Collection Runner / GraphQL
- [ ] **V3.0** — Mock Server / OpenAPI 导入 / 团队共享(可选)

---

## 🤝 贡献

欢迎 PR / Issue!

```bash
# 开发流程
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
