# SDKWork API Router

[中文文档](./README.zh-CN.md)

SDKWork API Router is a Rust-based OpenAI-compatible gateway, admin control plane, public portal, and product runtime. The repository ships both source-native development workflows and production-grade release/install tooling.

## Production Entry Points

Use these pages first when you are planning an online deployment:

- [Production Deployment](./docs/getting-started/production-deployment.md)
- [Install Layout](./docs/operations/install-layout.md)
- [Service Management](./docs/operations/service-management.md)
- [Docker And Helm Assets](./deploy/README.md)

For local development only, use:

- [Quickstart](./docs/getting-started/quickstart.md)
- [Source Development](./docs/getting-started/source-development.md)

## Runtime Surfaces

- `gateway-service`
  - OpenAI-compatible `/v1/*` gateway
- `admin-api-service`
  - operator-facing `/admin/*` control plane
- `portal-api-service`
  - developer-facing `/portal/*` self-service API
- `router-web-service`
  - Pingora-based public web host
- `router-product-service`
  - integrated production runtime serving `/admin/*`, `/portal/*`, and `/api/*`

## Configuration Contract

Primary config discovery order:

1. `router.yaml`
2. `router.yml`
3. `router.json`
4. `config.yaml`
5. `config.yml`
6. `config.json`

Effective precedence from lowest to highest:

- built-in defaults -> environment fallback -> config file -> CLI

Operational notes:

- `SDKWORK_CONFIG_DIR` and `SDKWORK_CONFIG_FILE` are discovery inputs.
- `conf.d/*.yaml` overlays load after the primary file in lexical order.
- system installs default to PostgreSQL.
- SQLite remains supported for local development and explicit portable validation flows.

## Deployment Modes

The release/install tooling supports two modes:

- `portable`
  - single-directory local validation and CI-friendly installs
- `system`
  - OS-standard production layout with external config, data, log, and run directories

`system` mode is the production standard.

## Recommended Production Flow

Build release artifacts:

```bash
./bin/build.sh
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

Generate a production-grade native install:

```bash
./bin/install.sh --mode system
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system
```

Validate the generated production config before service registration:

```bash
node bin/router-ops.mjs validate-config --mode system
```

```powershell
node .\bin\router-ops.mjs validate-config --mode system
```

Then continue with:

- [Production Deployment](./docs/getting-started/production-deployment.md) for Docker Compose, Helm, and native rollout guidance
- [Service Management](./docs/operations/service-management.md) for systemd, launchd, and Windows Service registration

## Local Development

Use the managed development entrypoints:

```bash
./bin/start-dev.sh
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1
```

The local development contract is documented in:

- [Quickstart](./docs/getting-started/quickstart.md)
- [Script Lifecycle](./docs/getting-started/script-lifecycle.md)

## Release And Verification

Release build/package guidance:

- [Release Builds](./docs/getting-started/release-builds.md)

Common verification baseline:

```bash
node --test scripts/check-router-docs-safety.test.mjs
node --test bin/tests/router-runtime-tooling.test.mjs
node --test scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs scripts/release/tests/deployment-assets.test.mjs
cargo test -p sdkwork-api-config --test config_loading
cargo test -p router-product-service
pnpm --dir docs build
```
