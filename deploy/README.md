# SDKWork API Router Deployment Assets

This page is Docker and Helm asset-specific.

For native `system` installs, OS-standard directories, service registration, and production initialization, use:

- [Production Deployment](../docs/getting-started/production-deployment.md)
- [Install Layout](../docs/operations/install-layout.md)
- [Service Management](../docs/operations/service-management.md)

## What Lives Here

- `docker/`
  - `Dockerfile`: Linux product-runtime image build
  - `docker-compose.yml`: quick PostgreSQL-backed single-host deployment
  - `.env.example`: required runtime secrets and database placeholders
- `helm/sdkwork-api-router/`
  - Kubernetes chart for externally managed PostgreSQL deployments

## Runtime Contract

These assets intentionally reuse the same production runtime contract:

- public web bind: `0.0.0.0:3001`
- internal gateway/admin/portal binds: `127.0.0.1:8080/8081/8082`
- bundled bootstrap data under `/opt/sdkwork/data`
- bundled admin and portal assets under `/opt/sdkwork/sites/*/dist`
- PostgreSQL-backed `SDKWORK_DATABASE_URL`

## Quick Docker Compose Deployment

```bash
tar -xzf sdkwork-api-router-product-server-linux-x64.tar.gz
cd sdkwork-api-router-product-server-linux-x64
cp deploy/docker/.env.example deploy/docker/.env
docker build -f deploy/docker/Dockerfile -t sdkwork-api-router:local .
docker compose -f deploy/docker/docker-compose.yml --env-file deploy/docker/.env up -d
```

## Quick Helm Deployment

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
