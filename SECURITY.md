# 安全说明 (Security)

ZeroApi 采用本地优先设计，默认情况下 API 数据不离开你的设备。

## 自动更新签名密钥

应用内自动更新使用 **minisign** 签名（`tauri-plugin-updater`）。

- **公钥**：已内置于 `src-tauri/tauri.conf.json` 的 `updater.pubkey`，随源代码提交，任何人可验证更新包签名。
- **私钥与密码**：位于 `src-tauri/.signing/`（**已被 .gitignore 忽略，严禁提交到仓库**）。
  - 一旦丢失私钥或密码，将永远无法再对更新包签名，自动更新将失效。
  - 请备份私钥到受控的密码管理器，并在 GitHub 仓库配置 Actions Secrets：
    - `TAURI_SIGNING_PRIVATE_KEY`：私钥文件内容
    - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`：私钥密码

> ⚠️ 如果这两个 Secret 未配置，CI 打包时无法生成 `latest.json`，自动更新会静默失败。

## 敏感数据

- 环境变量中的 **Secret** 使用 Windows DPAPI 加密后落库，明文不落库、不导出到项目文件。
- 请求与响应内容仅存储于本地 SQLite，不包含云端。

## 报告漏洞

请通过 GitHub Issue 提交（可见但仍建议先私信维护者），或在安全问题上开启一个 **draft** PR 讨论，不要直接公开可利用的细节。