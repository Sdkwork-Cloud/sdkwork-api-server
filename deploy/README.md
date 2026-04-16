# SDKWork API Router Deployment Assets

This directory contains the commercial deployment assets for the standalone
`router-product-service` runtime.

## Layout

- `docker/`
  - `Dockerfile`: builds a Linux runtime image from an extracted product-server bundle
  - `docker-compose.yml`: quick PostgreSQL-backed local or single-host deployment
  - `.env.example`: required runtime secrets and database defaults
- `helm/sdkwork-api-router/`
  - Helm chart for Kubernetes deployment with externally managed PostgreSQL

## Runtime Contract

All deployment assets reuse the existing server-mode runtime model:

- public web host binds `0.0.0.0:3001`
- internal gateway/admin/portal listeners stay on loopback `127.0.0.1:8080/8081/8082`
- bootstrap data is loaded from `/opt/sdkwork/data`
- static admin and portal assets are served from `/opt/sdkwork/sites/admin/dist`
  and `/opt/sdkwork/sites/portal/dist`
- `SDKWORK_DATABASE_URL` must point at PostgreSQL for production deployments

Use the Linux product-server release bundle as the Docker build context:

```bash
tar -xzf sdkwork-api-router-product-server-linux-x64.tar.gz
cd sdkwork-api-router-product-server-linux-x64
cp deploy/docker/.env.example deploy/docker/.env
docker build -f deploy/docker/Dockerfile -t sdkwork-api-router:local .
docker compose -f deploy/docker/docker-compose.yml --env-file deploy/docker/.env up -d
```

For Kubernetes, build and push the same image, then install the Helm chart:

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
