# sdkwork-api-server

[Chinese Guide](./README.zh-CN.md)

SDKWork API Server is an Axum-based OpenAI-compatible gateway, control plane, and public self-service portal built with Rust, React, pnpm, and Tauri.

The repository is organized around four runtime surfaces:

- `gateway-service`
  - OpenAI-compatible `/v1/*` gateway
- `admin-api-service`
  - operator-only `/admin/*` control plane
- `portal-api-service`
  - public `/portal/*` self-service registration, login, workspace inspection, and API key issuance
- `console/`
  - browser-accessible React shell that also runs inside the Tauri desktop host

## What Is Implemented

Backend:

- OpenAI-compatible gateway routes with stateful and stateless execution paths
- admin APIs for tenants, projects, API keys, channels, proxy providers, credentials, models, routing, usage, billing, and extensions
- public portal APIs:
  - `POST /portal/auth/register`
  - `POST /portal/auth/login`
  - `GET /portal/auth/me`
  - `GET /portal/workspace`
  - `GET /portal/api-keys`
  - `POST /portal/api-keys`
- shared SQLite and PostgreSQL persistence through one storage contract
- isolated portal JWT and admin JWT boundaries
- Prometheus-compatible metrics and HTTP tracing

Frontend:

- package-bounded portal SDK, portal auth, and portal dashboard modules
- package-bounded admin-facing console modules
- browser and Tauri-friendly hash routes:
  - `#/portal/register`
  - `#/portal/login`
  - `#/portal/dashboard`
  - `#/admin`

Architecture:

- controller or interface layer under `crates/sdkwork-api-interface-*`
- app or service layer under `crates/sdkwork-api-app-*`
- repository or storage layer under `crates/sdkwork-api-storage-*`
- shared React shell composition in `console/src/`
- reusable frontend business packages in `console/packages/`

## Repository Layout

```text
.
|-- crates/                      # domain, app, interface, provider, runtime, storage crates
|-- services/
|   |-- admin-api-service/       # standalone admin HTTP service
|   |-- gateway-service/         # standalone OpenAI-compatible gateway HTTP service
|   `-- portal-api-service/      # standalone public portal HTTP service
|-- console/                     # React + pnpm workspace + optional Tauri desktop shell
|-- scripts/
|   `-- dev/                     # cross-platform startup helpers
|-- docs/                        # architecture notes, plans, compatibility docs
`-- README.zh-CN.md              # Chinese operational guide
```

## Supported Platforms

This repository is intended to run on:

- Windows
- Linux
- macOS

The Rust services are cross-platform. The React console runs in a normal browser on all three platforms. The Tauri shell is optional and uses the same frontend routes, so desktop mode still keeps the browser UI reachable.

## Prerequisites

Required:

- Rust stable with Cargo
- Node.js 20+
- pnpm 10+

Optional:

- PostgreSQL 15+ for PostgreSQL-backed deployments
- Tauri CLI for desktop development:
  - `cargo install tauri-cli`

## Default Ports

| Surface | Default Bind | Purpose |
|---|---|---|
| gateway | `127.0.0.1:8080` | OpenAI-compatible `/v1/*` traffic |
| admin | `127.0.0.1:8081` | operator control plane |
| portal | `127.0.0.1:8082` | public self-service auth and API key lifecycle |
| console | `127.0.0.1:5173` | browser and Tauri frontend dev server |

## Preferred Startup Paths

Use the helper scripts below as the recommended entry points.

| Workflow | Windows | Linux / macOS |
|---|---|---|
| start backend services | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-servers.ps1` | `node scripts/dev/start-stack.mjs` |
| start browser console | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1` | `node scripts/dev/start-console.mjs` |
| start Tauri and keep browser access | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1 -Tauri` | `node scripts/dev/start-console.mjs --tauri` |
| preview production console build | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1 -Preview` | `node scripts/dev/start-console.mjs --preview` |

Notes:

- Windows service startup opens separate PowerShell windows for `admin-api-service`, `gateway-service`, and `portal-api-service`
- the Node-based helpers are portable and work on Windows, Linux, and macOS
- Tauri dev mode still exposes the browser UI on `http://127.0.0.1:5173`

## Quick Start With SQLite

This is the fastest end-to-end local setup.

### Windows

Terminal 1:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-servers.ps1
```

Terminal 2:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1
```

Open:

- `http://127.0.0.1:5173/#/portal/register`
- `http://127.0.0.1:5173/#/portal/login`
- `http://127.0.0.1:5173/#/portal/dashboard`
- `http://127.0.0.1:5173/#/admin`

### Linux or macOS

Terminal 1:

```bash
node scripts/dev/start-stack.mjs
```

Terminal 2:

```bash
node scripts/dev/start-console.mjs
```

Open the same browser URLs listed above.

## Public Portal Walkthrough

Once backend services and the console are running:

1. open `http://127.0.0.1:5173/#/portal/register`
2. register a portal account
3. land on `#/portal/dashboard`
4. create a gateway API key for `live`, `test`, or `staging`
5. copy the plaintext key immediately
6. call the gateway with that key

Example:

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer skw_live_your_key_here"
```

The portal list endpoint intentionally does not return plaintext keys again. Plaintext values are returned only at creation time.

## Browser and Tauri Together

If you want the desktop host and a normal browser open at the same time, use the Tauri startup helper.

### Windows

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1 -Tauri
```

### Linux or macOS

```bash
node scripts/dev/start-console.mjs --tauri
```

Why this works:

- `tauri dev` uses the Vite dev server as its frontend source
- the same Vite URL remains accessible from a normal browser
- portal registration, login, dashboard, and admin routes are the same in browser and Tauri

Open both if you want:

- desktop shell through Tauri
- browser on `http://127.0.0.1:5173/#/portal/dashboard`

## PostgreSQL Startup

To use PostgreSQL, point the services at the same PostgreSQL connection string.

### Windows

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-servers.ps1 `
  -DatabaseUrl "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

### Linux or macOS

```bash
node scripts/dev/start-stack.mjs \
  --database-url "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

SQLite and PostgreSQL migrations are applied automatically at startup.

## Raw Command Fallback

If you prefer to avoid helper scripts, these are the direct commands.

### Windows PowerShell

Admin:

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

Gateway:

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

Portal:

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p portal-api-service
```

Browser console:

```powershell
pnpm --dir console install
pnpm --dir console dev
```

Tauri:

```powershell
pnpm --dir console tauri:dev
```

### Linux or macOS

Admin:

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

Gateway:

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

Portal:

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p portal-api-service
```

Browser console:

```bash
pnpm --dir console install
pnpm --dir console dev
```

Tauri:

```bash
pnpm --dir console tauri:dev
```

## Browser Console

Install dependencies:

```bash
pnpm --dir console install
```

Run the browser dev server:

```bash
pnpm --dir console dev
```

Typecheck all packages:

```bash
pnpm --dir console -r typecheck
```

Build production assets:

```bash
pnpm --dir console build
```

Preview the production build locally:

```bash
pnpm --dir console preview
```

The dev server proxies these paths by default:

- `/admin` -> `http://127.0.0.1:8081`
- `/portal` -> `http://127.0.0.1:8082`
- `/v1` -> `http://127.0.0.1:8080`

Override them if needed:

- `SDKWORK_ADMIN_PROXY_TARGET`
- `SDKWORK_PORTAL_PROXY_TARGET`
- `SDKWORK_GATEWAY_PROXY_TARGET`

Example:

```powershell
$env:SDKWORK_ADMIN_PROXY_TARGET="http://127.0.0.1:18081"
$env:SDKWORK_PORTAL_PROXY_TARGET="http://127.0.0.1:18082"
$env:SDKWORK_GATEWAY_PROXY_TARGET="http://127.0.0.1:18080"
pnpm --dir console dev
```

## Service Health and Metrics

Health endpoints:

- gateway: `http://127.0.0.1:8080/health`
- admin: `http://127.0.0.1:8081/admin/health`
- portal: `http://127.0.0.1:8082/portal/health`

Metrics endpoints:

- gateway: `http://127.0.0.1:8080/metrics`
- admin: `http://127.0.0.1:8081/metrics`
- portal: `http://127.0.0.1:8082/metrics`

Example:

```bash
curl http://127.0.0.1:8082/portal/health
curl http://127.0.0.1:8082/metrics
```

## Runtime Configuration

Important environment variables:

- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_PORTAL_BIND`
- `SDKWORK_DATABASE_URL`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET`
- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_CREDENTIAL_MASTER_KEY`
- `SDKWORK_SECRET_LOCAL_FILE`
- `SDKWORK_SECRET_KEYRING_SERVICE`
- `SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS`
- `SDKWORK_EXTENSION_PATHS`
- `SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS`
- `SDKWORK_EXTENSION_TRUSTED_SIGNERS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS`

Supported secret backends:

- `database_encrypted`
- `local_encrypted_file`
- `os_keyring`

## What Is Still Intentionally Missing

The current system is usable end-to-end, but these areas remain roadmap work:

- multi-user portal workspaces and invitations
- password reset and email delivery
- OAuth or SSO
- standalone MySQL or libsql service startup

## Additional Reference Docs

Gateway compatibility and API surface coverage:

- `docs/api/compatibility-matrix.md`

Runtime topology and deployment notes:

- `docs/architecture/runtime-modes.md`

## Verification Commands

Current project-level verification baseline:

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console -r typecheck
pnpm --dir console build
```
