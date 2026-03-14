# sdkwork-api-server

[中文文档](./README.zh-CN.md)

SDKWork API Server is an Axum-based OpenAI-compatible gateway, control plane, public portal, and extension runtime built with Rust, React, pnpm, and Tauri.

## What This Repository Ships

Runtime surfaces:

- `gateway-service`
  - OpenAI-compatible `/v1/*` gateway
- `admin-api-service`
  - operator-only `/admin/*` control plane
- `portal-api-service`
  - public `/portal/*` self-service API
- `console/`
  - browser-accessible React shell that also runs inside Tauri
- `docs/`
  - VitePress documentation site with English and Chinese operational guides

Current foundations:

- Axum-based Rust services
- SQLite and PostgreSQL storage
- browser and Tauri console
- extension runtime for builtin, connector, and native-dynamic providers
- public portal registration, login, workspace inspection, and API key issuance

## Supported Platforms

- Windows
- Linux
- macOS

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

Key docs entry points:

- [Installation](./docs/getting-started/installation.md)
- [Source Development](./docs/getting-started/source-development.md)
- [Release Builds](./docs/getting-started/release-builds.md)
- [Runtime Modes](./docs/getting-started/runtime-modes.md)
- [Public Portal](./docs/getting-started/public-portal.md)
- [Configuration](./docs/operations/configuration.md)
- [Health and Metrics](./docs/operations/health-and-metrics.md)
- [API Compatibility](./docs/reference/api-compatibility.md)

Chinese docs entry points:

- [安装准备](./docs/zh/getting-started/installation.md)
- [源码运行](./docs/zh/getting-started/source-development.md)
- [Release 构建](./docs/zh/getting-started/release-builds.md)

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

- `http://127.0.0.1:5173/#/portal/register`
- `http://127.0.0.1:5173/#/portal/login`
- `http://127.0.0.1:5173/#/portal/dashboard`
- `http://127.0.0.1:5173/#/admin`

Lower-level source helpers:

- backend only:
  - `scripts/dev/start-servers.ps1`
  - `node scripts/dev/start-stack.mjs`
- console only:
  - `scripts/dev/start-console.ps1`
  - `node scripts/dev/start-console.mjs`

Detailed source instructions:

- [Source Development](./docs/getting-started/source-development.md)

## Release Build and Startup

Build release service binaries:

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service
```

Build browser console assets:

```bash
pnpm --dir console install
pnpm --dir console build
```

Build the Tauri desktop package:

```bash
pnpm --dir console tauri:build
```

Run release binaries with SQLite:

Windows:

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
.\target\release\admin-api-service.exe
```

Linux or macOS:

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
./target/release/admin-api-service
```

Start `gateway-service` and `portal-api-service` the same way from `target/release/`.

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

- `SDKWORK_DATABASE_URL`
- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_PORTAL_BIND`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET`
- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_EXTENSION_PATHS`

Detailed operational docs:

- [Configuration](./docs/operations/configuration.md)
- [Health and Metrics](./docs/operations/health-and-metrics.md)
- [Runtime Modes](./docs/getting-started/runtime-modes.md)

## Additional Technical References

- [Full Compatibility Matrix](./docs/api/compatibility-matrix.md)
- [Detailed Runtime Modes](./docs/architecture/runtime-modes.md)

## Verification

Current verification baseline:

```bash
pnpm --dir docs typecheck
pnpm --dir docs build
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console -r typecheck
pnpm --dir console build
```
