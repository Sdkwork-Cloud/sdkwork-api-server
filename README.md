# sdkwork-api-server

[中文说明](./README.zh-CN.md)

SDKWork API Server is an Axum-based OpenAI-compatible gateway, control plane, and public self-service portal built with Rust, React, pnpm, and Tauri.

It is designed around three HTTP boundaries plus one shared console:

- `gateway-service`: OpenAI-compatible `/v1/*` gateway
- `admin-api-service`: operator-only `/admin/*` control plane
- `portal-api-service`: public `/portal/*` registration, login, workspace, and API key self-service
- `console/`: browser-accessible React workspace that also runs inside the Tauri desktop shell

## Current Scope

Implemented and wired in the repository today:

- OpenAI-compatible gateway routes with stateful and stateless execution paths
- Admin APIs for tenants, projects, API keys, channels, proxy providers, credentials, models, routing, usage, billing, and extensions
- Public portal APIs for:
  - `POST /portal/auth/register`
  - `POST /portal/auth/login`
  - `GET /portal/auth/me`
  - `GET /portal/workspace`
  - `GET /portal/api-keys`
  - `POST /portal/api-keys`
- Shared SQLite and PostgreSQL persistence through the same storage abstraction
- Package-bounded React console modules for portal SDK, portal auth, portal user dashboard, and admin views
- Browser and Tauri-friendly hash routes:
  - `#/portal/register`
  - `#/portal/login`
  - `#/portal/dashboard`
  - `#/admin`
- Extension-oriented provider architecture with built-in, connector, and native dynamic runtimes

## Repository Layout

```text
.
|-- crates/                      # domain, app, interface, provider, runtime, storage crates
|-- services/
|   |-- admin-api-service/       # standalone admin HTTP service
|   |-- gateway-service/         # standalone OpenAI-compatible gateway HTTP service
|   `-- portal-api-service/      # standalone public portal HTTP service
|-- console/                     # React + pnpm workspace + optional Tauri desktop shell
|-- docs/                        # architecture notes, plans, compatibility docs
`-- README.zh-CN.md              # Chinese operational guide
```

## Architecture Summary

The backend follows a layered split:

- `interface/controller`
  - Axum routers under `crates/sdkwork-api-interface-*`
- `app/service`
  - orchestration, auth, key issuance, routing, and business flows under `crates/sdkwork-api-app-*`
- `repository/storage`
  - dialect-independent contracts in `crates/sdkwork-api-storage-core`
  - SQLite and PostgreSQL implementations in dedicated crates

The frontend follows the SDKWork package standard:

- root `console/src/` is a thin shell and route composition layer
- reusable modules live under `console/packages/`
- Tauri-specific logic stays under `console/src-tauri/`

## Supported Platforms

Startup and runtime are documented for:

- Windows with PowerShell
- Linux with bash
- macOS with zsh or bash

The Rust services are cross-platform. The web console runs in any browser with Node.js and pnpm. The Tauri shell is optional and uses the same console build.

## Prerequisites

Minimum recommended toolchain:

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

## Quick Start With SQLite

This is the fastest end-to-end local setup.

### Windows PowerShell

Open four terminals.

Terminal 1:

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

Terminal 2:

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

Terminal 3:

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p portal-api-service
```

Terminal 4:

```powershell
pnpm --dir console install
pnpm --dir console dev
```

Open:

- browser: `http://127.0.0.1:5173/#/portal/register`
- browser: `http://127.0.0.1:5173/#/portal/login`
- browser: `http://127.0.0.1:5173/#/portal/dashboard`
- browser: `http://127.0.0.1:5173/#/admin`

### Linux or macOS

Open four terminals.

Terminal 1:

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

Terminal 2:

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

Terminal 3:

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p portal-api-service
```

Terminal 4:

```bash
pnpm --dir console install
pnpm --dir console dev
```

Open the same browser URLs listed above.

## Public Portal Walkthrough

Once the four processes are running:

1. open `http://127.0.0.1:5173/#/portal/register`
2. register a portal account
3. land on `#/portal/dashboard`
4. create a gateway API key for `live`, `test`, or `staging`
5. copy the plaintext key immediately
6. call the gateway with that key

Example gateway call:

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer skw_live_your_key_here"
```

The portal list endpoint intentionally does not return plaintext keys again. Plaintext is only returned at creation time.

## PostgreSQL Startup

Use the same three-service flow, but point all services at the same PostgreSQL database URL.

### Windows PowerShell

```powershell
$env:SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p admin-api-service
```

In separate terminals:

```powershell
$env:SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p gateway-service
```

```powershell
$env:SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p portal-api-service
```

### Linux or macOS

```bash
export SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p admin-api-service
```

In separate terminals:

```bash
export SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p gateway-service
```

```bash
export SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p portal-api-service
```

SQLite and PostgreSQL migrations are applied automatically by the service startup path.

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

## Tauri Desktop Startup

The desktop shell lives under `console/src-tauri/`.

### Recommended local workflow

Terminal 1:

```bash
pnpm --dir console install
pnpm --dir console dev
```

Terminal 2:

```bash
cd console
pnpm tauri:dev
```

What this gives you:

- the Tauri app window
- the same Vite server still reachable in a normal browser
- shared hash routes for portal and admin views

That means desktop development does not block browser access. You can keep `http://127.0.0.1:5173/#/portal/dashboard` open in a browser while the same UI is also running in the desktop host.

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

## API Surface Notes

Gateway parity work is tracked in:

- `docs/api/compatibility-matrix.md`

Runtime modes and deployment notes are tracked in:

- `docs/architecture/runtime-modes.md`

## Verification Commands

These commands are the current project-level verification baseline:

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console -r typecheck
pnpm --dir console build
```

## Known Boundaries

Current intentional scope limits:

- one portal user owns one default tenant and one default project
- no portal invitations or multi-member workspace management yet
- no password reset or OAuth/SSO yet
- no MySQL or libsql standalone runtime support yet

Those are additive roadmap items, not blockers for the current standalone gateway + portal + browser/Tauri workflow.
