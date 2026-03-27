# Release Builds

This page covers how to produce and run deployable artifacts for services, browser assets, and optional desktop packages.

If you are looking for developer-oriented compilation commands, start with [Build and Packaging](/getting-started/build-and-packaging). This page focuses on release outputs.

## Release Build Targets

Standalone services:

- `admin-api-service`
- `gateway-service`
- `portal-api-service`
- `router-web-service`
- `router-product-service`

User-facing artifacts:

- admin app static assets
- portal web app static assets
- optional admin Tauri desktop package
- optional portal Tauri desktop package
- platform-specific product server bundle with `router-product-service` plus bundled admin and portal sites

## Build Rust Services

Build all release service binaries:

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

## Run Release Binaries

The standalone services resolve their configuration from the local SDKWork config root unless you override it with `SDKWORK_CONFIG_DIR` or `SDKWORK_CONFIG_FILE`.

### Windows

```powershell
New-Item -ItemType Directory -Force "$HOME\.sdkwork\router" | Out-Null
$env:SDKWORK_CONFIG_FILE="$HOME\.sdkwork\router\config.yaml"
.\target\release\admin-api-service.exe
```

```powershell
$env:SDKWORK_CONFIG_FILE="$HOME\.sdkwork\router\config.yaml"
.\target\release\gateway-service.exe
```

```powershell
$env:SDKWORK_CONFIG_FILE="$HOME\.sdkwork\router\config.yaml"
.\target\release\portal-api-service.exe
```

### Linux or macOS

```bash
mkdir -p "$HOME/.sdkwork/router"
export SDKWORK_CONFIG_FILE="$HOME/.sdkwork/router/config.yaml"
./target/release/admin-api-service
```

```bash
export SDKWORK_CONFIG_FILE="$HOME/.sdkwork/router/config.yaml"
./target/release/gateway-service
```

```bash
export SDKWORK_CONFIG_FILE="$HOME/.sdkwork/router/config.yaml"
./target/release/portal-api-service
```

## Build Admin App Assets

Install dependencies if needed:

```bash
pnpm --dir apps/sdkwork-router-admin install
```

Build:

```bash
pnpm --dir apps/sdkwork-router-admin build
```

The output goes to `apps/sdkwork-router-admin/dist/`.

You can host those assets with any static file server or CDN. During local verification, use:

```bash
pnpm --dir apps/sdkwork-router-admin preview
```

## Build Portal Web App Assets

Install dependencies if needed:

```bash
pnpm --dir apps/sdkwork-router-portal install
```

Build:

```bash
pnpm --dir apps/sdkwork-router-portal build
```

The output goes to `apps/sdkwork-router-portal/dist/`.

## Build the Tauri Desktop App

Desktop package build:

```bash
pnpm --dir apps/sdkwork-router-admin tauri:build
```

This produces OS-specific desktop artifacts under the Tauri build output directories.

## Release Deployment Notes

Recommended server-mode deployment shape:

- either run the standalone Rust services as independent processes
- run `router-web-service` in front of the admin and portal APIs when you want a unified public web entry
- or run `router-product-service` when you want one integrated server binary that serves `/admin/*`, `/portal/*`, and `/api/*`
- use PostgreSQL for durable multi-user deployments
- use a server-side secret backend strategy
- build `apps/sdkwork-router-admin/dist/` and `apps/sdkwork-router-portal/dist/`
- let `router-web-service` or `router-product-service` expose those static assets under `/admin/` and `/portal/`

## Automated GitHub Releases

`.github/workflows/release.yml` publishes tagged or manually triggered GitHub releases.

Native release automation now includes:

- Windows x64 and arm64
- Linux x64 and arm64
- macOS x64 and arm64
- admin, portal, and console desktop packages
- standalone service binaries
- `router-product-service`
- platform-specific product server bundles with bundled admin and portal sites

Recommended embedded-mode deployment shape:

- use the Tauri desktop shell
- keep SQLite local by default
- prefer OS keyring or local encrypted file secret storage

## Release Verification

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service -p router-product-service
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs build
```
