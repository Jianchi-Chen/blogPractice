# Tauri 自动更新配置指南

## 概述

本项目已配置 Tauri 自动更新功能，更新源为 GitHub Releases。用户可以通过系统托盘菜单点击"检查更新"来手动检查并安装更新。

---

## 工作原理

### 1. 更新检查流程

```
用户点击"检查更新"
    ↓
调用 tray.rs 中的 check_version()
    ↓
从 GitHub Release 下载 latest.json
    ↓
对比当前版本与最新版本
    ↓
下载更新包并验证签名
    ↓
安装更新（下次启动时生效）
```

### 2. 核心文件

- **[tray.rs](frontend/src-tauri/src/tray.rs)** - 托盘菜单和更新逻辑
- **[App.vue](frontend/src/App.vue)** - 监听更新事件并显示进度
- **[tauri.conf.json](frontend/src-tauri/tauri.conf.json)** - updater 配置
- **[.github/workflows/tauri-release.yml](.github/workflows/tauri-release.yml)** - CI/CD 自动构建和发布

---

## 配置详解

### 1. tauri.conf.json 配置

```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/Jianchi-Chen/myBlog/releases/latest/download/latest.json"
      ],
      "dialog": false,  // 使用自定义 UI，不弹系统对话框
      "active": true,
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6..."  // 公钥用于验证更新包签名
    }
  }
}
```

**说明：**
- `endpoints`: 指向 GitHub Release 上的 `latest.json` 文件
- `dialog: false`: 禁用默认对话框，使用 Naive UI 的消息提示
- `pubkey`: 由 `tauri signer generate` 生成的公钥（验证更新包完整性）

### 2. GitHub Secrets 配置

在 GitHub 仓库设置中添加以下 secrets：

| Secret 名称 | 说明 | 必需 |
|------------|------|-----|
| `TAURI_SIGNING_PRIVATE_KEY` | 签名私钥内容 | ✅ 是 |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 私钥密码（如果有） | ⚠️ 可选 |

**生成签名密钥：**

```bash
# 进入项目目录
cd frontend

# 生成密钥对（会保存到 ~/.tauri/ 目录）
npm run tauri signer generate -- --write-keys ~/.tauri/myblog.key

# 查看私钥内容（复制到 GitHub Secrets）
cat ~/.tauri/myblog.key

# 查看公钥（已配置在 tauri.conf.json 中）
cat ~/.tauri/myblog.key.pub
```

⚠️ **安全提示：** 永远不要提交私钥到 Git 仓库！

---

## CI/CD 构建流程

### 触发方式

#### 方式 1：推送版本 Tag

```bash
git tag v1.0.0
git push origin v1.0.0
```

#### 方式 2：手动触发

在 GitHub Actions 页面点击 "Run workflow"

### 构建产物

每个平台会生成以下文件：

| 平台 | 安装包 | 更新包 |
|------|--------|--------|
| Windows | `.msi` | `.msi.zip` (签名) |
| macOS | `.dmg`, `.app` | `.app.tar.gz` (签名) |
| Linux | `.deb`, `.rpm`, `.AppImage` | `.AppImage.tar.gz` (签名) |

### latest.json 生成

`tauri-action` 会自动生成 `latest.json` 文件并上传到 Release：

```json
{
  "version": "1.0.0",
  "notes": "更新说明...",
  "pub_date": "2024-12-25T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "...",
      "url": "https://github.com/.../app_1.0.0_x64.msi.zip"
    },
    "darwin-x86_64": {
      "signature": "...",
      "url": "https://github.com/.../app_x64.app.tar.gz"
    },
    "linux-x86_64": {
      "signature": "...",
      "url": "https://github.com/.../app_amd64.AppImage.tar.gz"
    }
  }
}
```

---

## 更新流程详解

### 用户侧流程

1. **触发检查**：用户右键点击系统托盘图标 → "检查更新"
2. **检查中**：显示 "正在检查更新" 消息
3. **找到更新**：显示 "检查到更新，正在下载..."
4. **下载中**：实时显示下载进度（0-100%）
5. **下载完成**：显示 "更新下载完成，正在安装"
6. **安装完成**：下次启动应用时自动应用更新

### 前端事件监听

[App.vue](frontend/src/App.vue) 监听 `update_status` 事件：

```typescript
type UpdateStatusPayload = {
  status: "checking" | "found" | "downloading" | "none" | "error" | "done";
  progress?: number;
  message?: string;
};
```

### 后端实现

[tray.rs](frontend/src-tauri/src/tray.rs) 实现：

```rust
async fn check_version(app: &AppHandle) -> Result<(), String> {
    let updater = app.updater()?;
    let update = updater.check().await?;
    
    if let Some(update) = update {
        update.download_and_install(
            |downloaded, total| { /* 进度回调 */ },
            || { /* 完成回调 */ }
        ).await?;
    }
}
```

---

## 版本号管理

### 自动同步版本号

CI/CD 会从 Git tag 中提取版本号并自动更新三个配置文件：

1. `frontend/src-tauri/tauri.conf.json` → `"version": "1.0.0"`
2. `frontend/src-tauri/Cargo.toml` → `version = "1.0.0"`
3. `frontend/package.json` → `"version": "1.0.0"`

### 版本号格式

遵循 [Semantic Versioning](https://semver.org/)：

- `v1.0.0` - 稳定版本
- `v1.0.0-beta` - Beta 测试版
- `v1.0.0-alpha.1` - Alpha 测试版

---

## 测试更新功能

### 本地测试

1. **打包当前版本**：
   ```bash
   cd frontend
   npm run tauri:build
   ```

2. **修改版本号并打包新版本**：
   ```bash
   # 修改 tauri.conf.json 中的 version
   # 再次打包
   npm run tauri:build
   ```

3. **手动上传到 GitHub Release**：
   - 创建 Release（例如 `v1.0.1`）
   - 上传新版本的更新包（`.msi.zip` 等）
   - 手动创建 `latest.json` 文件

### 生产环境测试

1. 推送 tag 触发 CI/CD：
   ```bash
   git tag v1.0.1
   git push origin v1.0.1
   ```

2. 等待 CI/CD 完成构建

3. 安装旧版本应用，点击"检查更新"测试

---

## 常见问题

### Q1: 更新检查失败，提示签名验证错误

**原因：** 公钥与私钥不匹配

**解决：**
1. 重新生成密钥对：
   ```bash
   npm run tauri signer generate -- --write-keys ~/.tauri/myblog.key
   ```
2. 更新 `tauri.conf.json` 中的 `pubkey`
3. 更新 GitHub Secrets 中的 `TAURI_SIGNING_PRIVATE_KEY`

### Q2: latest.json 文件未生成

**原因：** CI/CD 配置缺少关键参数

**解决：** 确保 `.yml` 中包含：
```yaml
includeUpdaterJson: true
updaterJsonKeepUniversal: true
```

### Q3: 更新下载后无法安装

**原因：**
- Windows: 可能被杀毒软件拦截
- macOS: 需要用户授权（Gatekeeper）
- Linux: 需要可执行权限

**解决：**
- 为应用添加代码签名证书（macOS/Windows）
- 使用 `notarization`（macOS）

### Q4: 用户如何回退到旧版本？

**方法 1：** 从 GitHub Releases 下载旧版本安装包重新安装

**方法 2：** 在 CI/CD 中保留最近 2 个 Release（已配置）

### Q5: 如何禁用自动更新？

修改 `tauri.conf.json`：
```json
{
  "plugins": {
    "updater": {
      "active": false  // 设置为 false
    }
  }
}
```

---

## 进阶配置

### 1. 启动时自动检查更新

在 [lib.rs](frontend/src-tauri/src/lib.rs) 的 `setup` 中添加：

```rust
.setup(|app| {
    // 启动后延迟 5 秒检查更新
    let app_handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        if let Err(e) = check_version(&app_handle).await {
            log::warn!("自动检查更新失败: {}", e);
        }
    });
    
    Ok(())
})
```

### 2. 自定义更新 UI

修改 [App.vue](frontend/src/App.vue) 中的 `UpdateStatusListener` 组件：

```vue
<template>
  <n-modal v-model:show="showUpdateDialog">
    <n-card title="发现新版本">
      <p>{{ updateMessage }}</p>
      <n-progress :percentage="downloadProgress" />
    </n-card>
  </n-modal>
</template>
```

### 3. 多环境更新源

```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://api.example.com/updates/{{target}}/{{current_version}}",
        "https://github.com/user/repo/releases/latest/download/latest.json"
      ]
    }
  }
}
```

---

## 参考文档

- [Tauri Updater 官方文档](https://v2.tauri.app/plugin/updater/)
- [tauri-action GitHub](https://github.com/tauri-apps/tauri-action)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [代码签名最佳实践](https://v2.tauri.app/distribute/sign/)

---

## 维护清单

- [ ] 定期检查 `TAURI_SIGNING_PRIVATE_KEY` 是否泄露
- [ ] 每次发布前测试更新功能
- [ ] 监控 GitHub Actions 构建日志
- [ ] 定期清理旧版本 Release（保留最近 2-3 个）
- [ ] 更新 `latest.json` endpoint URL（如果仓库重命名）

---

**最后更新时间**：2024-12-25  
**作者**：GitHub Copilot  
**项目版本**：v0.1.0
