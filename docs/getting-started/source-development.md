# Source Development

This page documents the recommended source-based startup flows for Windows, Linux, and macOS.

## Default Ports

| Surface | Default Bind | Purpose |
|---|---|---|
| gateway | `127.0.0.1:8080` | OpenAI-compatible `/v1/*` traffic |
| admin | `127.0.0.1:8081` | operator control plane |
| portal | `127.0.0.1:8082` | public auth, workspace, and API key lifecycle |
| console | `127.0.0.1:5173` | browser and Tauri frontend dev server |

## Fastest End-to-End Startup

### Windows

Browser mode:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1
```

Desktop mode:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri
```

### Linux or macOS

Browser mode:

```bash
node scripts/dev/start-workspace.mjs
```

Desktop mode:

```bash
node scripts/dev/start-workspace.mjs --tauri
```

## Partial Startup

Backend services only:

```bash
node scripts/dev/start-stack.mjs
```

Browser console only:

```bash
node scripts/dev/start-console.mjs
```

Desktop console only:

```bash
node scripts/dev/start-console.mjs --tauri
```

Windows PowerShell wrappers are also available:

- `scripts/dev/start-servers.ps1`
- `scripts/dev/start-console.ps1`
- `scripts/dev/start-workspace.ps1`

## SQLite Development

SQLite is the default local database:

- `SDKWORK_DATABASE_URL=sqlite://sdkwork-api-server.db`

No extra setup is required. Database creation and migrations happen on startup.

## PostgreSQL Development

Use a shared PostgreSQL connection string across admin, gateway, and portal:

```bash
node scripts/dev/start-workspace.mjs \
  --database-url "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 `
  -DatabaseUrl "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

## Raw Source Commands

Run the Rust services directly:

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p portal-api-service
```

Run the browser console:

```bash
pnpm --dir console dev
```

Run Tauri from source:

```bash
pnpm --dir console tauri:dev
```

## Recommended Verification

Before or after local startup, the standard checks are:

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console -r typecheck
pnpm --dir console build
pnpm --dir docs typecheck
pnpm --dir docs build
```
