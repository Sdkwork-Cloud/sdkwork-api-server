# Release Builds

This page covers how to build and run release artifacts for services, browser assets, and optional desktop packages.

## Release Build Targets

Backend services:

- `admin-api-service`
- `gateway-service`
- `portal-api-service`

Frontend targets:

- browser console static assets
- optional Tauri desktop package

## Build Rust Services

Build all release service binaries:

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service
```

The output binaries are placed under `target/release/`.

Windows executable names:

- `target/release/admin-api-service.exe`
- `target/release/gateway-service.exe`
- `target/release/portal-api-service.exe`

Linux and macOS executable names:

- `target/release/admin-api-service`
- `target/release/gateway-service`
- `target/release/portal-api-service`

## Run Release Binaries

### Windows

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
.\target\release\admin-api-service.exe
```

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
.\target\release\gateway-service.exe
```

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
.\target\release\portal-api-service.exe
```

### Linux or macOS

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
./target/release/admin-api-service
```

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
./target/release/gateway-service
```

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
./target/release/portal-api-service
```

## Build Browser Console Assets

Install dependencies if needed:

```bash
pnpm --dir console install
```

Build:

```bash
pnpm --dir console build
```

The output goes to `console/dist/`.

You can host those assets with any static file server or CDN. During local verification, use:

```bash
pnpm --dir console preview
```

## Build the Tauri Desktop App

Desktop package build:

```bash
pnpm --dir console tauri:build
```

This produces OS-specific desktop artifacts under the Tauri build output directories.

## Release Deployment Notes

Recommended server-mode deployment shape:

- run the three Rust services as independent processes
- use PostgreSQL for durable multi-user deployments
- use a server-side secret backend strategy
- host `console/dist/` separately if you need a browser-facing console

Recommended embedded-mode deployment shape:

- use the Tauri desktop shell
- keep SQLite local by default
- prefer OS keyring or local encrypted file secret storage

## Release Verification

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service
pnpm --dir console build
pnpm --dir docs build
```
