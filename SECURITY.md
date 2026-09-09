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

### Secret 加密策略

| 平台 | 加密方案 | 密钥存储位置 |
|------|---------|-------------|
| Windows | DPAPI（CryptProtectData / CryptUnprotectData，CURRENT_USER scope） | 由操作系统管理，绑定当前用户账户 |
| macOS / Linux | XChaCha20-Poly1305 对称加密 | 本地密钥文件 `~/.config/com.zeroapi.app/.zeroapi_key` |

- **加密失败行为**：返回错误，拒绝存储，**不会回退到明文**。
- **解密失败行为**：UI 显示 `[无法解密: 可能因系统变更或密钥丢失]`，原始加密数据保留在数据库中，不会被清空。
- **跨平台迁移限制**：
  - Windows DPAPI 密文绑定当前用户 + 当前机器，迁移数据库文件到新机器后无法解密。
  - macOS/Linux 密钥文件独立存储，迁移时需同时复制密钥文件，否则无法解密。

### 数据存储范围

- 环境变量中的 **Secret** 加密后落库，明文不落库、不导出到项目文件。
- 导出收藏夹时，敏感请求头（Authorization、Cookie、X-Api-Key 等）和鉴权信息会被脱敏为 `***`。
- 请求与响应内容仅存储于本地 SQLite，不包含云端。

### 已知限制

- **非 Windows 平台依赖本地密钥文件**：若密钥文件被删除或损坏，此前加密的 Secret 将无法恢复。
- **TLS 证书验证**：默认启用证书验证；仅在用户于设置中关闭 "Verify SSL" 时，诊断探测和代理探测才会跳过证书检查，并会在日志中输出警告。

## 报告漏洞

请通过 GitHub Issue 提交（可见但仍建议先私信维护者），或在安全问题上开启一个 **draft** PR 讨论，不要直接公开可利用的细节。
