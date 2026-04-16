# Release Builds

This page covers how to produce and run deployable artifacts for services, browser assets, and optional desktop packages.

If you need the full responsibility matrix for every script, start with [Script Lifecycle](/getting-started/script-lifecycle). This page focuses on release outputs and deployment flow.

## Recommended Managed Release Flow

The recommended release lifecycle is:

1. `bin/build.*`
2. `bin/install.*`
3. review `config/router.env`
4. `bin/start.*`
5. verify the unified and direct URLs printed at startup
6. `bin/stop.*` or hand off to a service manager

### Build

Linux or macOS:

```bash
./bin/build.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

This compiles:

- Rust release binaries
- admin and portal static assets
- docs build output
- optional desktop bundles
- the native release package under `artifacts/release/`

### Install

Linux or macOS:

```bash
./bin/install.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1
```

By default, the install home is:

- `artifacts/install/sdkwork-api-router/current/`

Important install-home paths:

- `bin/`
- `config/router.env`
- `sites/admin/dist`
- `sites/portal/dist`
- `var/log/`
- `var/run/`
- `service/systemd/`
- `service/launchd/`
- `service/windows-task/`

### Configure

Review or override:

- `config/router.env`

This is the supported place to change:

- release bind addresses
- database location
- static site directories
- proxy targets

### Start

Linux or macOS:

```bash
./bin/start.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1
```

The managed release runtime starts `router-product-service`, which serves:

- `/admin/*`
- `/portal/*`
- `/api/*`

By default, it binds:

- gateway: `127.0.0.1:8080`
- admin: `127.0.0.1:8081`
- portal: `127.0.0.1:8082`
- unified web host: `0.0.0.0:3001`

After successful startup, the scripts print:

- unified admin URL
- unified portal URL
- unified gateway health URL
- direct service URLs
- seeded local admin and portal credentials
- log file locations

### Stop

Linux or macOS:

```bash
./bin/stop.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\stop.ps1
```

## Foreground And Service-Manager Use

For systemd, launchd, Windows Task Scheduler, or any other service manager, use foreground mode:

- `bin/start.sh --foreground`
- `bin/start.ps1 -Foreground`

The install step already stages service registration assets:

- `service/systemd/`
- `service/launchd/`
- `service/windows-task/`
- `deploy/docker/`
- `deploy/helm/sdkwork-api-router/`

Register or unregister them from the install home:

- Linux / systemd:
  - `./service/systemd/install-service.sh`
  - `./service/systemd/uninstall-service.sh`
- macOS / launchd:
  - `./service/launchd/install-service.sh`
  - `./service/launchd/uninstall-service.sh`
- Windows / Task Scheduler:
  - `powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-task\install-service.ps1 -StartNow`
  - `powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-task\uninstall-service.ps1`

## Script Roles In The Release Lifecycle

| Script | Role in lifecycle | Important note |
|---|---|---|
| `bin/build.*` | creates releasable binaries and assets | does not install or start |
| `bin/install.*` | prepares the runnable install home | does not start the runtime |
| `bin/start.*` | starts the installed release runtime | assumes install home already exists |
| `bin/stop.*` | stops the installed release runtime | uses the install-home PID file |

## Lower-Level Release Commands

Use the raw commands below when you intentionally want manual control instead of the managed release lifecycle.

### Build Rust Services

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service
cargo build --release -p router-web-service -p router-product-service
```

The output binaries are placed under `target/release/`.

### Output paths

Windows executable names:

- `target/release/admin-api-service.exe`
- `target/release/gateway-service.exe`
- `target/release/portal-api-service.exe`
- `target/release/router-web-service.exe`
- `target/release/router-product-service.exe`

Linux and macOS executable names:

- `target/release/admin-api-service`
- `target/release/gateway-service`
- `target/release/portal-api-service`
- `target/release/router-web-service`
- `target/release/router-product-service`

### Build Admin App Assets

```bash
pnpm --dir apps/sdkwork-router-admin install
pnpm --dir apps/sdkwork-router-admin build
```

Output:

- `apps/sdkwork-router-admin/dist/`

### Build Portal Web App Assets

```bash
pnpm --dir apps/sdkwork-router-portal install
pnpm --dir apps/sdkwork-router-portal build
```

Output:

- `apps/sdkwork-router-portal/dist/`

### Build The Tauri Desktop App

```bash
pnpm --dir apps/sdkwork-router-admin tauri:build
```

## Release Deployment Notes

Recommended deployment shape:

- use `router-product-service` when you want one integrated runtime serving `/admin/*`, `/portal/*`, and `/api/*`
- use PostgreSQL for durable multi-user deployments
- use a server-side secret backend strategy
- keep `config/router.env` under change control for environment-specific overrides
- use the managed install home instead of inventing a second runtime layout

### Docker And Kubernetes Assets

The Linux product-server bundle now includes deployable assets under `deploy/`:

- `deploy/docker/Dockerfile`
- `deploy/docker/docker-compose.yml`
- `deploy/docker/.env.example`
- `deploy/helm/sdkwork-api-router/`

These assets intentionally reuse the product server runtime contract instead of introducing a
second deployment model:

- public web bind: `0.0.0.0:3001`
- internal upstream binds: `127.0.0.1:8080/8081/8082`
- bootstrap data directory: `/opt/sdkwork/data`
- bundled admin site directory: `/opt/sdkwork/sites/admin/dist`
- bundled portal site directory: `/opt/sdkwork/sites/portal/dist`
- production database: `SDKWORK_DATABASE_URL=postgresql://...`

Quick Docker deployment from an extracted Linux bundle:

```bash
cp deploy/docker/.env.example deploy/docker/.env
docker build -f deploy/docker/Dockerfile -t sdkwork-api-router:local .
docker compose -f deploy/docker/docker-compose.yml --env-file deploy/docker/.env up -d
```

Quick Helm deployment after pushing the same image:

```bash
helm upgrade --install sdkwork-api-router deploy/helm/sdkwork-api-router \
  --set image.repository=ghcr.io/your-org/sdkwork-api-router \
  --set image.tag=2026.04.15 \
  --set secrets.databaseUrl='postgresql://sdkwork:change-me@postgresql:5432/sdkwork_api_router' \
  --set secrets.adminJwtSigningSecret='change-me-admin' \
  --set secrets.portalJwtSigningSecret='change-me-portal' \
  --set secrets.credentialMasterKey='change-me-master-key' \
  --set secrets.metricsBearerToken='change-me-metrics-token'
```

## Dry-Run Examples

```bash
./bin/build.sh --dry-run
./bin/install.sh --dry-run
./bin/start.sh --dry-run
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 --dry-run
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 --dry-run
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1 -DryRun
```

## Release Verification

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service -p router-product-service
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs build
```

## Next Steps

- script responsibilities and lifecycle:
  - [Script Lifecycle](/getting-started/script-lifecycle)
- source startup flows:
  - [Source Development](/getting-started/source-development)
- build pipeline details:
  - [Build and Packaging](/getting-started/build-and-packaging)
