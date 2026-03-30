# 数据库初始化问题修复说明

## 问题描述

1. **Linux (Ubuntu) 和可能的 RedHat 系统**：Tauri 桌面应用无法启动或打开
2. **Windows 11**：下载发布的 Tauri 应用后无法登录超管用户

## 根本原因分析

### 原因 1：种子数据文件路径问题
- 原代码使用相对路径 `./seeds` 读取种子数据文件
- 在开发环境工作正常，但打包后该目录不存在
- 导致应用启动时无法创建超管用户

### 原因 2：文件系统权限不足
- Tauri 应用的 capabilities 配置中缺少写入权限
- 无法在用户数据目录创建数据库文件

### 原因 3：数据库初始化错误处理不完善
- 初始化失败时错误信息不够明确
- 日志输出不足，难以定位问题

## 修复措施

### 1. 种子数据嵌入 (Tauri)

**文件**: `frontend/src-tauri/src/db.rs`

**修改**：
- 使用 `include_str!` 宏将种子数据嵌入到二进制文件中
- 添加 `should_run_seeds` 函数检查用户表是否为空
- 只在首次创建数据库时执行种子数据，避免重复执行

```rust
// 嵌入种子数据
const SEED_SUPERUSER: &str = include_str!("../seeds/0001_superuser.sql");

// 检查是否需要运行种子数据
async fn should_run_seeds(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(count.0 == 0)
}
```

### 2. 文件系统权限修复

**文件**: `frontend/src-tauri/capabilities/default.json`

**添加权限**：
- `fs:allow-write-file` - 允许写入文件
- `fs:allow-create` - 允许创建文件
- `fs:allow-mkdir` - 允许创建目录

### 3. 改进错误处理和日志

**文件**: `frontend/src-tauri/src/lib.rs`

**修改**：
- 添加详细的初始化步骤日志
- 改进错误消息，明确指出失败的具体步骤
- 在每个关键步骤添加日志输出

```rust
log::info!("=== Application Initialization Started ===");
log::info!("Creating database connection pool...");
log::info!("Running database migrations...");
log::info!("Checking and running seeds if needed...");
log::info!("=== Application Initialization Completed Successfully ===");
```

### 4. 跨平台路径处理

**文件**: `frontend/src-tauri/src/config.rs`

**修改**：
- 统一使用正斜杠路径（SQLite 在所有平台都支持）
- 添加更详细的路径日志输出
- 确保目录创建失败时有明确的错误日志

### 5. Backend 同步修复

**文件**: `backend/src/db/mod.rs`

**修改**：
- 同样将种子数据嵌入到二进制文件
- 添加用户表检查逻辑
- 改进日志输出

## 测试建议

### Windows 11
1. 删除现有的应用数据目录：`%APPDATA%\io.jianchi.blog`
2. 重新打包应用：`npm run tauri:build`
3. 安装并运行打包的应用
4. 使用默认账号登录：
   - 用户名：`admin`
   - 密码：`tfF;1J(2WokG,5`

### Linux (Ubuntu)
1. 删除现有数据：`~/.local/share/io.jianchi.blog/`
2. 重新编译：`npm run tauri:build`
3. 运行应用并检查日志
4. 测试登录功能

### 日志查看
- 应用启动时会在终端输出详细的初始化日志
- 检查以下关键信息：
  - App data directory 路径
  - Database URL
  - Migrations 执行状态
  - Seeds 执行状态

## 预期结果

1. ✅ 应用在所有平台都能正常创建数据库文件
2. ✅ 用户数据目录会自动创建
3. ✅ 超管用户在首次运行时自动创建
4. ✅ 重复运行不会重复创建用户
5. ✅ 初始化失败时有明确的错误提示

## 相关文件清单

### Tauri 应用
- `frontend/src-tauri/src/db.rs` - 数据库连接和种子数据加载
- `frontend/src-tauri/src/lib.rs` - 应用初始化流程
- `frontend/src-tauri/src/config.rs` - 配置和路径管理
- `frontend/src-tauri/capabilities/default.json` - 文件系统权限配置

### Backend 服务器
- `backend/src/db/mod.rs` - 数据库初始化（同步修复）

## 版本信息

- Tauri: 2.9.2
- SQLx: 0.8
- 修复日期: 2026-03-30
