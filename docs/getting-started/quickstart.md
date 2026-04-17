# Quickstart

This page is for local development only.

If you need an online deployment, system install, PostgreSQL-backed runtime, or service-manager rollout, use [Production Deployment](/getting-started/production-deployment).

## Prerequisites

- Rust + Cargo
- Node.js 20+
- pnpm 10+

## Start The Managed Development Runtime

Linux or macOS:

```bash
./bin/start-dev.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1
```

Default managed development URLs:

- admin: `http://127.0.0.1:9983/admin/`
- portal: `http://127.0.0.1:9983/portal/`
- gateway health: `http://127.0.0.1:9983/api/v1/health`

## Verify Local Health

```bash
curl http://127.0.0.1:9980/health
curl http://127.0.0.1:9981/admin/health
curl http://127.0.0.1:9982/portal/health
```

Each endpoint should return `ok`.

## Stop The Managed Development Runtime

Linux or macOS:

```bash
./bin/stop-dev.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\stop-dev.ps1
```

## Next Steps

- [Source Development](/getting-started/source-development)
- [Script Lifecycle](/getting-started/script-lifecycle)
- [Production Deployment](/getting-started/production-deployment)
