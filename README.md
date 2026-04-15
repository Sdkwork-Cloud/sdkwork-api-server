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
  - [Script Lifecycle](./docs/getting-started/script-lifecycle.md)
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
  - [脚本生命周期](./docs/zh/getting-started/script-lifecycle.md)
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
| preview mode | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Preview` | `node scripts/dev/start-workspace.mjs --preview` |
| desktop mode | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri` | `node scripts/dev/start-workspace.mjs --tauri` |
| dry run | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -DryRun` | `node scripts/dev/start-workspace.mjs --dry-run` |

Open:

- browser mode admin app: `http://127.0.0.1:5173/admin/`
- browser mode portal app: `http://127.0.0.1:5174/portal/`
- desktop or preview web host: `http://127.0.0.1:9983/portal/`
- desktop or preview admin site: `http://127.0.0.1:9983/admin/`

Notes:

- the source workspace helpers now default to `127.0.0.1:9980` for gateway, `127.0.0.1:9981` for admin, `127.0.0.1:9982` for portal, and `0.0.0.0:9983` for the shared web host.
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

## Managed Bin Scripts

The repository now includes a deployment-oriented `bin/` script set that sits above the lower-level
`scripts/dev/*` helpers.

For repository ergonomics, root-level start/build/install/stop scripts are compatibility wrappers that delegate to `bin/*`, while the `bin/*` scripts remain the managed source of truth for lifecycle behavior.

If you want one page that explains the full `build -> install -> start -> verify -> stop -> service registration`
lifecycle, read [Script Lifecycle](./docs/getting-started/script-lifecycle.md) first.

Recommended uses:

- `scripts/dev/*`
  - direct source workflows when you want the original repo-native startup helpers
- `bin/start-dev.sh` / `bin/start-dev.ps1`
  - managed development startup with a writable local SQLite database under `artifacts/runtime/dev/`
  - default dev binds moved to the `998x` range to avoid the common `808x` collisions:
    - gateway: `127.0.0.1:9980`
    - admin: `127.0.0.1:9981`
    - portal: `127.0.0.1:9982`
    - shared web host: `127.0.0.1:9983`
  - defaults to preview mode so the built-in Pingora web host is the primary entrypoint; use `--browser` or `-Browser` when you explicitly want the standalone Vite admin and portal frontends instead
  - prints a formatted startup summary with unified web URLs, direct service URLs, log files, and the seeded admin / portal credentials
  - the underlying dev launchers now supervise nested child processes and wait for them to stop before exiting, so `Ctrl+C`, `bin/stop-dev.*`, and child crash handling are less likely to leave orphaned `pnpm` or `cargo` processes behind
  - on Windows, `start-dev.sh` delegates to `start-dev.ps1`, so Git Bash / MSYS paths do not leak into the Windows Node runtime; the command name stays the same, but the lifecycle implementation is unified with PowerShell
- `bin/build.sh` / `bin/build.ps1`
  - release-oriented build pipeline for:
    - Rust release binaries
    - admin and portal browser assets
    - console and docs browser assets
    - admin and portal Tauri release bundles
  - native release package output under `artifacts/release/`
  - on Windows, the build and install pipeline automatically use a managed short cargo target directory when `CARGO_TARGET_DIR` is not set, reducing MSVC/CMake path-length failures
  - on Windows, release builds default to `CARGO_BUILD_JOBS=1` when you do not set it explicitly, which matches the most reliable MSVC/CMake path validated for this workspace; override `CARGO_BUILD_JOBS` if you intentionally want a different concurrency trade-off
  - on Windows, `build.sh` delegates to `build.ps1`, so Git Bash users follow the same verified build path as native PowerShell users
- `bin/install.sh` / `bin/install.ps1`
  - install the release runtime into `artifacts/install/sdkwork-api-router/current` by default
  - copy release binaries plus admin and portal static sites
  - stage only the production runtime assets in the install home; `build.*`, `install.*`, and `start-dev.*` remain source-tree tools
  - generate:
    - `config/router.env`
    - `service/systemd/sdkwork-api-router.service`
    - `service/systemd/install-service.sh`
    - `service/systemd/uninstall-service.sh`
    - `service/launchd/com.sdkwork.api-router.plist`
    - `service/launchd/install-service.sh`
    - `service/launchd/uninstall-service.sh`
    - `service/windows-task/sdkwork-api-router.xml`
    - `service/windows-task/install-service.ps1`
    - `service/windows-task/uninstall-service.ps1`
  - on Windows, `install.sh` delegates to `install.ps1`, keeping path handling and install behavior aligned with the verified PowerShell implementation
- `bin/start.sh` / `bin/start.ps1`
  - production runtime entrypoint
  - starts `router-product-service` in release mode, serving `/admin/*`, `/portal/*`, and `/api/*`
  - uses a writable local SQLite database under the installed runtime `var/data/` directory by default
  - prints a formatted startup summary with unified web URLs, direct service URLs, log files, and the seeded admin / portal credentials
  - designed for direct daemon use or for foreground service-manager execution
  - on Windows, `start.sh` delegates to `start.ps1`, keeping the production shell entrypoint available from Git Bash without reintroducing MSYS path translation bugs
- `bin/stop.sh` / `bin/stop.ps1`
  - stop the managed production runtime
  - the PowerShell `start/stop` helpers now resolve platform-specific binary names and process-stop behavior at runtime, so the same `pwsh` entrypoints remain usable on Linux and macOS installs in addition to Windows

Typical release flow:

Linux or macOS:

```bash
./bin/build.sh
./bin/install.sh
./bin/start.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1
```

Dry-run examples:

```bash
./bin/build.sh --dry-run
./bin/install.sh --dry-run
./bin/start-dev.sh --dry-run
./bin/start.sh --dry-run
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 --dry-run
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 --dry-run
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -DryRun
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1 -DryRun
```

Important runtime notes:

- `bin/start.sh --foreground` and `bin/start.ps1 -Foreground` are the service-manager-friendly forms.
- both `bin/start-dev.*` and `bin/start.*` print clickable local URLs for the unified web entrypoint plus the direct backend ports after successful startup.
- `bin/start-dev.*` and `bin/start.*` share the `9980`-series defaults. If another runtime is already bound to those ports, the scripts now fail fast before spawning child services and print the conflicting bind addresses directly instead of surfacing a late health-check timeout or "started then exited" symptom.
- the managed start scripts now keep a companion `.state.env` file next to each managed pid file. It records the active bind set, frontend mode, and process fingerprint so repeat `start` / `stop` calls can distinguish a healthy managed instance from a stale or PID-reused process.
- if a managed runtime is already healthy on a different bind set, re-running `bin/start-dev.*` or `bin/start.*` now prints the active managed addresses instead of failing with a generic health-check error.
- on Windows, the `.sh` wrappers are compatibility entrypoints that hand off to the corresponding `.ps1` scripts. This keeps Git Bash workflows available while avoiding `/d/...` MSYS path mismatches inside Windows Node / cargo processes.
- `bin/start-dev.*` and `bin/start.*` now print bootstrap identity guidance instead of fixed demo credentials.
- development identities come from the active bootstrap profile; for the local `dev` bootstrap profile, review `data/identities/dev.json` before sharing the environment, and note that the default `prod` bootstrap profile does not seed development identities.
- `bin/install.*` writes native service descriptors plus register/unregister helper scripts, but still does not auto-register them during a generic install run.
- the installed runtime home intentionally includes production `start/stop` scripts and service-management assets only.
- the generated `config/router.env` is the primary place to override release binds, database location, and site directories.
- `config/router.env` now quotes values so installed paths with spaces remain safe for both the shell helpers and `systemd` environment loading.
- on Windows, `bin/install.*` resolves release binaries from the same managed short cargo target directory used by `bin/build.*` unless you explicitly provide `CARGO_TARGET_DIR`.
- development scripts intentionally avoid the default user-home SQLite path that can become read-only in constrained environments.

Daemon registration examples from the installed runtime home:

Linux / systemd:

```bash
./service/systemd/install-service.sh
./service/systemd/uninstall-service.sh
```

macOS / launchd:

```bash
./service/launchd/install-service.sh
./service/launchd/uninstall-service.sh
```

Windows / Task Scheduler:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-task\install-service.ps1 -StartNow
powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-task\uninstall-service.ps1
```

## Release Build and Startup

For the recommended managed release flow, prefer the `bin/build.*`, `bin/install.*`, `bin/start.*`,
and `bin/stop.*` scripts above. The commands in this section remain useful when you want the
individual lower-level build steps.

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

Typical managed-script endpoints:

- unified admin app: `http://127.0.0.1:9983/admin/`
- unified portal app: `http://127.0.0.1:9983/portal/`
- unified gateway health: `http://127.0.0.1:9983/api/v1/health`
- direct gateway health: `http://127.0.0.1:9980/health`
- direct admin health: `http://127.0.0.1:9981/admin/health`
- direct portal health: `http://127.0.0.1:9982/portal/health`

The standalone binaries still honor the built-in local config defaults documented earlier in this README when you do not override their bind addresses.

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
