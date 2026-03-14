# sdkwork-api-server

[English Guide](./README.md)

SDKWork API Server 是一个基于 Axum 的 OpenAI 兼容网关、控制平面、公共门户和扩展运行时系统，技术栈包括 Rust、React、pnpm 和 Tauri。

## 当前仓库包含什么

运行面：

- `gateway-service`
  - OpenAI 兼容 `/v1/*` 网关
- `admin-api-service`
  - 面向 operator 的 `/admin/*` 控制平面
- `portal-api-service`
  - 面向外部用户的 `/portal/*` 自助 API
- `console/`
  - 可在浏览器访问，也可运行在 Tauri 中的 React 控制台
- `docs/`
  - 使用 VitePress 构建的中英文文档站

当前基础能力：

- 基于 Axum 的 Rust 服务
- SQLite 与 PostgreSQL 存储
- 浏览器与 Tauri 双运行形态
- builtin、connector、native-dynamic 扩展运行时
- public portal 注册、登录、工作区查看与 API key 签发

## 支持平台

- Windows
- Linux
- macOS

## 文档站

本地安装并预览 docs：

```bash
pnpm --dir docs install
pnpm --dir docs dev
```

构建 docs：

```bash
pnpm --dir docs build
```

关键英文文档入口：

- [Installation](./docs/getting-started/installation.md)
- [Source Development](./docs/getting-started/source-development.md)
- [Release Builds](./docs/getting-started/release-builds.md)
- [Runtime Modes](./docs/getting-started/runtime-modes.md)
- [Public Portal](./docs/getting-started/public-portal.md)
- [Configuration](./docs/operations/configuration.md)
- [Health and Metrics](./docs/operations/health-and-metrics.md)

关键中文文档入口：

- [安装准备](./docs/zh/getting-started/installation.md)
- [源码运行](./docs/zh/getting-started/source-development.md)
- [Release 构建](./docs/zh/getting-started/release-builds.md)
- [运行模式](./docs/zh/getting-started/runtime-modes.md)
- [Public Portal](./docs/zh/getting-started/public-portal.md)
- [配置说明](./docs/zh/operations/configuration.md)
- [健康检查与 Metrics](./docs/zh/operations/health-and-metrics.md)

## 环境要求

必需：

- Rust stable 与 Cargo
- Node.js 20+
- pnpm 10+

可选：

- PostgreSQL 15+
- Tauri CLI：

```bash
cargo install tauri-cli
```

## 源码启动

推荐完整栈启动方式：

| 工作流 | Windows | Linux / macOS |
|---|---|---|
| 浏览器模式 | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1` | `node scripts/dev/start-workspace.mjs` |
| 桌面模式 | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri` | `node scripts/dev/start-workspace.mjs --tauri` |
| dry run | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -DryRun` | `node scripts/dev/start-workspace.mjs --dry-run` |

打开：

- `http://127.0.0.1:5173/#/portal/register`
- `http://127.0.0.1:5173/#/portal/login`
- `http://127.0.0.1:5173/#/portal/dashboard`
- `http://127.0.0.1:5173/#/admin`

更底层的源码辅助脚本：

- 仅后端：
  - `scripts/dev/start-servers.ps1`
  - `node scripts/dev/start-stack.mjs`
- 仅控制台：
  - `scripts/dev/start-console.ps1`
  - `node scripts/dev/start-console.mjs`

完整说明见：

- [源码运行](./docs/zh/getting-started/source-development.md)

## Release 构建与启动

构建 release 服务二进制：

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service
```

构建浏览器前端资源：

```bash
pnpm --dir console install
pnpm --dir console build
```

构建 Tauri 桌面包：

```bash
pnpm --dir console tauri:build
```

使用 SQLite 启动 release 二进制：

Windows：

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
.\target\release\admin-api-service.exe
```

Linux 或 macOS：

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
./target/release/admin-api-service
```

`gateway-service` 与 `portal-api-service` 也使用同样方式从 `target/release/` 启动。

完整说明见：

- [Release 构建](./docs/zh/getting-started/release-builds.md)

## 运行与运维

关键端点：

- gateway health: `http://127.0.0.1:8080/health`
- admin health: `http://127.0.0.1:8081/admin/health`
- portal health: `http://127.0.0.1:8082/portal/health`
- gateway metrics: `http://127.0.0.1:8080/metrics`
- admin metrics: `http://127.0.0.1:8081/metrics`
- portal metrics: `http://127.0.0.1:8082/metrics`

关键环境变量：

- `SDKWORK_DATABASE_URL`
- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_PORTAL_BIND`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET`
- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_EXTENSION_PATHS`

详细运维文档：

- [配置说明](./docs/zh/operations/configuration.md)
- [健康检查与 Metrics](./docs/zh/operations/health-and-metrics.md)
- [运行模式](./docs/zh/getting-started/runtime-modes.md)

## 额外技术参考

- [完整兼容矩阵](./docs/api/compatibility-matrix.md)
- [运行模式详解](./docs/architecture/runtime-modes.md)

## 验证

当前验证基线：

```bash
pnpm --dir docs typecheck
pnpm --dir docs build
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console -r typecheck
pnpm --dir console build
```
