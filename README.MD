# myBlog

> **项目声明**：这是一个用于学习、练手和验证技术方案的个人项目，代码与接口可能随时调整，也不承诺持续维护、兼容性、可用性或数据安全。请勿直接用于生产环境或保存重要数据。仓库中的默认账户和发布产物仅用于开发体验，不代表已经达到实际部署标准。

myBlog 是一个 Vue 3、Axum、SQLite 与 Tauri 2 组合的全栈博客练习项目。Web 与桌面端共用同一套 Vue 前端和 Axum HTTP API；Tauri 当前定位为轻量桌面壳，负责系统托盘、自动更新和桌面交付，不再内嵌另一套业务后端或数据库。

## 当前架构

```text
Web 浏览器 ─┐
            ├─ Vue 3 前端 ── HTTP/JSON ── Axum API ── SQLite
Tauri 桌面 ─┘       │
                    └─ 桌面端托盘与签名更新
```

- `backend/`：独立 Rust API 服务，使用 Axum、SQLx 和 SQLite，负责鉴权、文章、评论、用户资料、收藏与管理接口。
- `frontend/`：Vue 3 + TypeScript 应用，通过统一 HTTP 客户端访问后端，可构建为 Web 页面或 Tauri 前端。
- `frontend/src-tauri/`：Tauri 2 桌面壳，仅保留桌面集成、托盘和签名更新逻辑。
- `docs/`：API 契约、部署迁移和桌面发布说明。

## 已实现功能

- 文章列表、详情、搜索建议与 Markdown 编辑/预览
- JWT 登录注册、角色权限、个人资料与头像
- 用户收藏、评论与幂等点赞
- 文章和用户管理、草稿及发布状态管理
- Web 与 Tauri 共用业务 API
- Tauri 系统托盘、启动检查更新、签名下载与安装后重启
- SQLite 迁移、种子数据和前后端契约测试

这些功能主要用于练习完整链路，不应理解为生产可用性承诺。

## 技术栈

| 范围 | 主要技术 |
| --- | --- |
| 前端 | Vue 3、TypeScript、Vite、Naive UI、Tailwind CSS、Pinia、Vue Router、Vditor |
| 后端 | Rust、Axum、SQLx、SQLite、JWT、Argon2 |
| 桌面 | Tauri 2、系统托盘、签名自动更新 |
| 测试 | Vitest、MSW、Rust 单元测试与 API 契约测试 |

## 本地开发

需要 Node.js 20 或更高版本、Rust stable，以及当前平台要求的 Tauri 系统依赖。

### 1. 启动后端

先将 `backend/.env.example` 复制为 `backend/.env`，并填写至少 32 字节的随机 `JWT_SECRET`。不要提交真实密钥。

```bash
cd backend
cargo run
```

默认监听 `http://127.0.0.1:3000`，数据库迁移和开发种子会在启动时执行。

### 2. 启动 Web 前端

```bash
cd frontend
npm ci
npm run dev
```

开发服务器默认将 `/api` 请求代理到 `http://127.0.0.1:3000`。需要连接其他服务时，可通过 `VITE_API_BASE_URL` 指定 HTTP(S) 源站，不要附加 `/api` 路径。

### 3. 启动 Tauri

先保持后端运行，再执行：

```bash
cd frontend
npm ci
npm run tauri:dev
```

桌面正式构建必须提供 `VITE_API_BASE_URL`：

```powershell
$env:VITE_API_BASE_URL = "https://api.example.com"
npm run tauri:build
```

## 检查与测试

```bash
cd frontend
npm run test
npm run build

cd ../backend
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test

cd ../frontend/src-tauri
cargo test
```

## 开发种子账户

首次初始化空数据库时会创建公开的开发账户：

- 用户名：`admin`
- 密码：`tfF;1J(2WokG,5`

该凭据已公开在仓库中，只能用于本地练习。不要将带有此账户的数据库或服务暴露到公网。

## 文档与发布

- [文档索引](docs/README.md)
- [HTTP API 契约](docs/api-contract.md)
- [部署与迁移](docs/deployment-and-migration.md)
- [桌面发布与签名更新](docs/desktop-release.md)

推送 `vX.Y.Z` 形式的标签会触发桌面发布工作流。各平台产物先上传到草稿 Release，全部构建成功后才公开。可在 [GitHub Releases](https://github.com/Jianchi-Chen/myBlog/releases) 查看产物，但这些产物同样仅供学习和测试。

## 项目边界

- 不保证生产级安全、性能、稳定性和升级兼容性
- 不保证默认配置适合公网部署
- 不为使用本项目造成的数据丢失或其他风险负责
- 接口、数据库和桌面发布流程可能继续调整
- 欢迎将代码作为学习参考，但请自行审查和验证

## 许可证

当前未提供开源许可证。除非另行授权，仓库公开不代表授予复制、分发或生产使用许可。
