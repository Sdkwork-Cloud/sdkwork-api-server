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

From the build/release tooling environment, run:

```bash
node bin/router-ops.mjs validate-config --mode system
```

```powershell
node .\bin\router-ops.mjs validate-config --mode system
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
