# Production Deployment

This is the canonical production deployment guide for SDKWork API Router.

Use this page when you are publishing online, preparing a native server install, building a Docker Compose deployment, or rolling out a Helm release.

## Production Contract

- `system` install mode is the native production standard.
- PostgreSQL is the default database contract for `system` installs.
- Config files are the primary source of truth.
- Environment variables are discovery inputs and fallback values.
- Service supervision belongs to `systemd`, `launchd`, or Windows Service Control Manager.

## Choose A Deployment Path

### Docker Compose

Use this when you want the fastest single-host rollout with PostgreSQL included.

Primary assets:

- `deploy/docker/Dockerfile`
- `deploy/docker/docker-compose.yml`
- `deploy/docker/.env.example`

### Helm

Use this when you want Kubernetes deployment with externally managed PostgreSQL.

Primary assets:

- `deploy/helm/sdkwork-api-router/`

### Native System Install

Use this when you need an OS-standard installation with service-manager startup.

## Build Release Artifacts

Linux or macOS:

```bash
./bin/build.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

Cross-platform release hygiene:

- keep Windows-only `CMAKE_GENERATOR` and `HOST_CMAKE_GENERATOR` settings scoped to Windows entrypoints and CI jobs
- do not persist Visual Studio CMake generator defaults in global Cargo config or Unix shell profiles
- when you run Unix installed-runtime smoke inside Docker, keep the same `CARGO_TARGET_DIR` for the `cargo build` and `run-unix-installed-runtime-smoke.mjs` steps
- the runtime starts correctly even when `ss`, `netstat`, and `lsof` are unavailable; install one of them when you want richer bind-conflict diagnostics during preflight

## Local Release Governance Preparation

If you run release governance from a development host where sibling repositories are not clean standalone release checkouts, point the release tooling at a managed external dependency root first. This keeps `materialize-external-deps`, `verify-release-sync`, and `run-release-governance-checks` aligned to governed clones instead of unrelated local worktrees.

Linux or macOS:

```bash
export SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_ROOT="$PWD/artifacts/external-release-deps"
node scripts/release/materialize-external-deps.mjs
node scripts/release/verify-release-sync.mjs --format text --live
node scripts/release/run-release-governance-checks.mjs
```

Windows:

```powershell
$env:SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_ROOT = (Join-Path (Get-Location) 'artifacts\external-release-deps')
node scripts/release/materialize-external-deps.mjs
node scripts/release/verify-release-sync.mjs --format text --live
node scripts/release/run-release-governance-checks.mjs
```

Use this whenever direct sibling audits report reasons such as `not-standalone-root`, `dirty-working-tree`, `branch-not-synced`, or `head-mismatch`.

## Generate A Native Production Install

Linux or macOS:

```bash
./bin/install.sh --mode system
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system
```

Generated production assets include:

- canonical `router.yaml`
- `conf.d/` overlay directory
- `router.env`
- `router.env.example`
- service descriptors for `systemd`, `launchd`, and Windows Service

## Initialize Production Configuration

Edit the generated runtime config before first start:

- `router.yaml`
  - canonical runtime config
- `conf.d/*.yaml`
  - optional domain-specific overlays
- `router.env`
  - discovery values and minimal runtime fallback values

Recommended first edits:

- replace the PostgreSQL placeholder with a real database URL
- set JWT, credential, and metrics secrets
- review bind addresses and trusted network boundaries
- confirm admin and portal static site locations

## Validate Before Service Registration

From the installed runtime home, run:

```bash
./bin/validate-config.sh
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\validate-config.ps1
```

If you are validating a generated install from the build/release repository instead of from the installed runtime home, you can still run:

```bash
node bin/router-ops.mjs validate-config --mode system --home <install-root>
```

```powershell
node .\bin\router-ops.mjs validate-config --mode system --home <install-root>
```

Validation checks:

- config discovery and merge order
- production security posture
- rejection of SQLite in `system` mode unless an explicit development override is enabled

## Register And Start Services

Use foreground entrypoints under a service manager:

- Linux: `./service/systemd/install-service.sh`
- macOS: `./service/launchd/install-service.sh`
- Windows: `powershell -NoProfile -ExecutionPolicy Bypass -File .\service\windows-service\install-service.ps1`

Reference guides:

- [Install Layout](/operations/install-layout)
- [Service Management](/operations/service-management)

## Docker Compose Quick Deployment

```bash
cp deploy/docker/.env.example deploy/docker/.env
docker build -f deploy/docker/Dockerfile -t sdkwork-api-router:local .
docker compose -f deploy/docker/docker-compose.yml --env-file deploy/docker/.env up -d
```

## Helm Quick Deployment

```bash
helm upgrade --install sdkwork-api-router deploy/helm/sdkwork-api-router \
  --set image.repository=ghcr.io/your-org/sdkwork-api-router \
  --set image.tag=2026.04.15 \
  --set secrets.databaseUrl='postgresql://sdkwork:change-me@postgresql:5432/sdkwork_api_router'
```

## Initialization Checklist

- release bundle built for the target platform
- PostgreSQL database created and reachable
- `router.yaml` reviewed
- `router.env` secrets replaced
- `validate-config` run successfully
- service registered through the OS-native manager
- health endpoints verified after first start
