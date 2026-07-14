# Changelog

All notable changes to ZeroApi will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-07-26

### 🎉 First Public Release

ZeroApi V1.0 正式发布 - 零登录、零云端、零 CORS 的轻量级 API 调试工具。

### ✨ Features

- **HTTP 七种方法**: GET / POST / PUT / PATCH / DELETE / HEAD / OPTIONS
- **完整请求配置**: Query 参数 / Headers / Body (form-data / x-www-form-urlencoded / raw JSON/XML/Text) / Auth (None / Bearer / Basic / API Key)
- **响应展示**: 状态码 + 耗时 + 大小 + Body + Headers + Cookies
- **JSON 美化高亮**: 自动识别 Content-Type + Shiki 语法高亮 + Pretty/Raw 切换
- **cURL 导入**: 解析 cURL 命令(支持 -X / -H / -d / -F / -u / -G 等常见 flag)
- **历史记录**: 自动保存最近 100 条,可清空
- **收藏夹**: ⭐ 一键收藏当前请求,命名保存
- **集合管理**: 创建集合,组织多个请求
- **环境变量**: `{{var}}` 占位符,多环境切换
- **导入/导出 JSON**: 完整数据备份还原
- **设置**: 主题(浅色/深色/跟随系统)、超时、代理、SSL、自动保存
- **应用内自动更新**: Tauri Updater 框架,GitHub Releases 一键升级

### 🎨 UI

- 薄荷绿主色(#7FC8A9),清新简约
- Inter / PingFang SC / JetBrains Mono 字体
- 8-12px 圆角,充足留白
- 自研 shadcn 风格组件
- Monaco Editor 请求体编辑
- Shiki 响应高亮

### 🏗️ 技术

- Tauri 2 + Rust + WebView2
- React 18 + TypeScript + Vite
- Tailwind CSS
- SQLite (rusqlite, bundled)
- reqwest + rustls(无 CORS 限制)
- Zustand 状态管理
- React Router 路由
- 启动 < 2s,安装包 < 10MB

### 🔒 Privacy

- ✅ 零登录(不收集身份)
- ✅ 零云端(数据全本地 SQLite)
- ✅ 零追踪(无任何 SDK)
- ✅ 零 CORS(Rust reqwest 直连)

### 📦 Download

- `ZeroApi_1.0.0_x64-setup.exe` - NSIS 安装版
- `ZeroApi_1.0.0_x64-setup.msi` - MSI 安装版
- `ZeroApi_1.0.0_x64.exe` - 便携版

### ⚠️ Known Limitations

- 未做代码签名(Win11 首次运行会提示"未知发布者",点"仍要运行"即可)
- V1.0 不支持 macOS / Linux
- 暂无暗色模式自适应图标(亮色图标在深色任务栏略浅)

---

## [Unreleased]

### V1.1 Plan

- 暗色模式完善
- 响应 Preview(HTML 渲染 / 图片预览)
- Header 模板
- 完整快捷键大全(`Ctrl+K` 命令面板)
- 代码签名
