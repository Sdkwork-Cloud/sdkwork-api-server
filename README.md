# sdkwork-api-server

[中文文档](./README.zh-CN.md)

SDKWork API Server is an Axum-based OpenAI-compatible gateway, control plane, public portal, and extension runtime built with Rust, React, pnpm, and Tauri.

## What This Repository Ships

Runtime surfaces:

- `gateway-service`
  - OpenAI-compatible `/v1/*` gateway
- `admin-api-service`
  - operator-facing `/admin/*` control plane
- `portal-api-service`
  - public `/portal/*` self-service API
- `router-web-service`
  - Pingora-based public web host for `/admin/*`, `/portal/*`, and API proxy entrypoints
- `apps/sdkwork-router-admin/`
  - standalone super-admin browser app plus the admin-owned Tauri desktop host
- `apps/sdkwork-router-portal/`
  - standalone developer self-service portal app
- `docs/`
  - VitePress documentation site with English and Chinese operational guides

Current foundations:

- Axum-based Rust services
- SQLite and PostgreSQL storage
- Pingora-backed public web delivery for admin and portal
- standalone browser admin and portal apps
- admin-owned Tauri desktop host
- extension runtime for builtin, connector, and native-dynamic providers
- public portal registration, login, dashboard, usage, billing posture, and API key issuance
- local JSON and YAML runtime configuration under `~/.sdkwork/router/`

## Supported Platforms

- Windows
- Linux
- macOS

## Quick Start

The standalone services now support a local config directory with built-in defaults.

Default config root:

- Linux and macOS: `~/.sdkwork/router/`
- Windows: `%USERPROFILE%\\.sdkwork\\router\\`

Config file discovery order:

1. `config.yaml`
2. `config.yml`
3. `config.json`

Config precedence:

1. built-in local defaults
2. local config file
3. `SDKWORK_*` environment variables

If no config file exists, the server still starts with local defaults:

- gateway bind: `127.0.0.1:8080`
- admin bind: `127.0.0.1:8081`
- portal bind: `127.0.0.1:8082`
- SQLite database: `~/.sdkwork/router/sdkwork-api-server.db`
- extension directory: `~/.sdkwork/router/extensions`
- local secrets file: `~/.sdkwork/router/secrets.json`

Example `config.yaml`:

```yaml
gateway_bind: "127.0.0.1:8080"
admin_bind: "127.0.0.1:8081"
portal_bind: "127.0.0.1:8082"
database_url: "sqlite://sdkwork-api-server.db"
secret_backend: "local_encrypted_file"
secret_local_file: "secrets.json"
extension_paths:
  - "extensions"
enable_connector_extensions: true
enable_native_dynamic_extensions: false
```

Relative paths inside config files are resolved relative to the config file directory.

To override the default location:

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`

## Docs

Install and preview the docs site locally:

```bash
pnpm --dir docs install
pnpm --dir docs dev
```

Build the docs site:

```bash
pnpm --dir docs build
```

Primary English doc structure:

- Getting Started:
  - [Quickstart](./docs/getting-started/quickstart.md)
  - [Installation](./docs/getting-started/installation.md)
  - [Source Development](./docs/getting-started/source-development.md)
  - [Build and Packaging](./docs/getting-started/build-and-packaging.md)
  - [Release Builds](./docs/getting-started/release-builds.md)
- Architecture:
  - [Software Architecture](./docs/architecture/software-architecture.md)
  - [Functional Modules](./docs/architecture/functional-modules.md)
  - [Runtime Modes Deep Dive](./docs/architecture/runtime-modes.md)
- API Reference:
  - [Overview](./docs/api-reference/overview.md)
  - [Gateway API](./docs/api-reference/gateway-api.md)
  - [Admin API](./docs/api-reference/admin-api.md)
  - [Portal API](./docs/api-reference/portal-api.md)
- Operations:
  - [Configuration](./docs/operations/configuration.md)
  - [Health and Metrics](./docs/operations/health-and-metrics.md)
- Reference:
  - [API Compatibility](./docs/reference/api-compatibility.md)
  - [Repository Layout](./docs/reference/repository-layout.md)
  - [Build and Tooling](./docs/reference/build-and-tooling.md)

Primary Chinese doc structure:

- 开始使用：
  - [快速开始](./docs/zh/getting-started/quickstart.md)
  - [安装准备](./docs/zh/getting-started/installation.md)
  - [源码运行](./docs/zh/getting-started/source-development.md)
  - [编译与打包](./docs/zh/getting-started/build-and-packaging.md)
  - [发布构建](./docs/zh/getting-started/release-builds.md)
- 架构：
  - [软件架构](./docs/zh/architecture/software-architecture.md)
  - [功能模块](./docs/zh/architecture/functional-modules.md)
  - [运行模式详解](./docs/zh/architecture/runtime-modes.md)
- API 参考：
  - [总览](./docs/zh/api-reference/overview.md)
  - [网关 API](./docs/zh/api-reference/gateway-api.md)
  - [管理端 API](./docs/zh/api-reference/admin-api.md)
  - [门户 API](./docs/zh/api-reference/portal-api.md)
- 运维：
  - [配置说明](./docs/zh/operations/configuration.md)
  - [健康检查与 Metrics](./docs/zh/operations/health-and-metrics.md)
- 参考：
  - [API 兼容矩阵](./docs/zh/reference/api-compatibility.md)
  - [仓库结构](./docs/zh/reference/repository-layout.md)
  - [构建与工具链](./docs/zh/reference/build-and-tooling.md)

## Prerequisites

Required:

- Rust stable with Cargo
- Node.js 20+
- pnpm 10+

Optional:

- PostgreSQL 15+
- Tauri CLI:

```bash
cargo install tauri-cli
```

## Source Startup

Recommended full-stack startup:

| Workflow | Windows | Linux / macOS |
|---|---|---|
| browser mode | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1` | `node scripts/dev/start-workspace.mjs` |
| desktop mode | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri` | `node scripts/dev/start-workspace.mjs --tauri` |
| dry run | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -DryRun` | `node scripts/dev/start-workspace.mjs --dry-run` |

Open:

- browser mode admin app: `http://127.0.0.1:5173/admin/`
- browser mode portal app: `http://127.0.0.1:5174/portal/`
- desktop or preview web host: `http://127.0.0.1:3001/portal/`
- desktop or preview admin site: `http://127.0.0.1:3001/admin/`

Notes:

- `start-workspace --tauri` starts the admin desktop shell and the shared Pingora web host for external browser access.
- `start-workspace --preview` builds admin and portal, then serves both sites through the Pingora web host.

To start with a specific config root:

Windows:

```powershell
$env:SDKWORK_CONFIG_DIR="$HOME\\.sdkwork\\router"
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1
```

Linux or macOS:

```bash
export SDKWORK_CONFIG_DIR="$HOME/.sdkwork/router"
node scripts/dev/start-workspace.mjs
```

Lower-level source helpers:

- backend only:
  - `scripts/dev/start-servers.ps1`
  - `node scripts/dev/start-stack.mjs`
- admin only:
  - `node scripts/dev/start-admin.mjs`
- portal only:
  - `node scripts/dev/start-portal.mjs`
- public web host only:
  - `node scripts/dev/start-web.mjs`

Detailed source instructions:

- [Source Development](./docs/getting-started/source-development.md)

## Release Build and Startup

Build release service binaries:

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service
```

Build admin app assets:

```bash
pnpm --dir apps/sdkwork-router-admin install
pnpm --dir apps/sdkwork-router-admin build
```

Build standalone portal assets:

```bash
pnpm --dir apps/sdkwork-router-portal install
pnpm --dir apps/sdkwork-router-portal build
```

Build the Tauri desktop package:

```bash
pnpm --dir apps/sdkwork-router-admin tauri:build
```

Run release binaries with the default local config root:

Windows:

```powershell
New-Item -ItemType Directory -Force "$HOME\\.sdkwork\\router" | Out-Null
.\target\release\admin-api-service.exe
.\target\release\gateway-service.exe
.\target\release\portal-api-service.exe
```

Linux or macOS:

```bash
mkdir -p "$HOME/.sdkwork/router"
./target/release/admin-api-service
./target/release/gateway-service
./target/release/portal-api-service
```

Run with an explicit config file:

Windows:

```powershell
$env:SDKWORK_CONFIG_FILE="$HOME\\.sdkwork\\router\\config.yaml"
.\target\release\gateway-service.exe
```

Linux or macOS:

```bash
export SDKWORK_CONFIG_FILE="$HOME/.sdkwork/router/config.yaml"
./target/release/gateway-service
```

Environment variables such as `SDKWORK_DATABASE_URL` still override file values.

Detailed release instructions:

- [Release Builds](./docs/getting-started/release-builds.md)

## Runtime and Operations

Important endpoints:

- gateway health: `http://127.0.0.1:8080/health`
- admin health: `http://127.0.0.1:8081/admin/health`
- portal health: `http://127.0.0.1:8082/portal/health`
- gateway metrics: `http://127.0.0.1:8080/metrics`
- admin metrics: `http://127.0.0.1:8081/metrics`
- portal metrics: `http://127.0.0.1:8082/metrics`

Important environment variables:

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`
- `SDKWORK_DATABASE_URL`
- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_PORTAL_BIND`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET`
- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_SECRET_LOCAL_FILE`
- `SDKWORK_EXTENSION_PATHS`

Detailed operational docs:

- [Configuration](./docs/operations/configuration.md)
- [Health and Metrics](./docs/operations/health-and-metrics.md)
- [Runtime Modes](./docs/getting-started/runtime-modes.md)
- [Software Architecture](./docs/architecture/software-architecture.md)
- [API Reference Overview](./docs/api-reference/overview.md)

## Additional Technical References

- [Full Compatibility Matrix](./docs/api/compatibility-matrix.md)
- [Detailed Runtime Modes](./docs/architecture/runtime-modes.md)

## Gateway Protocol Compatibility

The gateway now exposes translated compatibility layers for existing agent clients without creating a second routing system.

- Claude Code and other Anthropic Messages clients can call `POST /v1/messages` and `POST /v1/messages/count_tokens`.
- Gemini CLI gateway mode and other Google Generative Language clients can call `POST /v1beta/models/{model}:generateContent`, `POST /v1beta/models/{model}:streamGenerateContent?alt=sse`, and `POST /v1beta/models/{model}:countTokens`.
- Stateful gateway deployments accept the existing `Authorization: Bearer ...` header plus compatibility-native auth inputs: `x-api-key` for Anthropic-style clients, and `x-goog-api-key` or `?key=` for Gemini-style clients.
- These compatibility routes translate into the existing OpenAI-compatible chat and token-count execution path, so routing policy selection, quota checks, billing, usage recording, and upstream provider relay stay shared with the `/v1/*` gateway surface.

## Verification

Current verification baseline:

```bash
pnpm --dir docs typecheck
pnpm --dir docs build
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir apps/sdkwork-router-admin typecheck
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
```
