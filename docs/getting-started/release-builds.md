# Release Builds

This page covers build and package generation only.

If you want to deploy online, initialize PostgreSQL, register services, or use the OS-standard `system` layout, use [Production Deployment](/getting-started/production-deployment).

## Managed Build Pipeline

Linux or macOS:

```bash
./bin/build.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

The managed build produces:

- Rust release binaries
- admin and portal browser assets
- docs build output
- optional desktop bundles
- native release output under `artifacts/release/`

## Managed Install Package Generation

Portable install package generation:

```bash
./bin/install.sh
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1
```

Production-grade native install generation:

```bash
./bin/install.sh --mode system
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system
```

## Dry Run

```bash
./bin/build.sh --dry-run
./bin/install.sh --dry-run
./bin/install.sh --mode system --dry-run
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 --dry-run
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 --dry-run
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system -DryRun
```

## Verification

```bash
node --test bin/tests/router-runtime-tooling.test.mjs
node --test scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs scripts/release/tests/deployment-assets.test.mjs
```

## Next Steps

- [Production Deployment](/getting-started/production-deployment)
- [Install Layout](/operations/install-layout)
- [Service Management](/operations/service-management)
