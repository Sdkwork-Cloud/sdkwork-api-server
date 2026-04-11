# 2026-04-11 Install Deploy Bootstrap Verification Step Update

## What Changed

- Tightened PowerShell managed-runtime state parsing in `bin/lib/runtime-common.ps1` so incomplete or stale state files no longer crash under `Set-StrictMode`.
- Added repository-level shell line-ending protection via `.gitattributes`:
  - `*.sh text eol=lf`
- Normalized repository shell entrypoints to LF so `bin/start.sh` and related install/runtime helpers execute correctly from WSL and Unix shells.
- Packaged repository `/data` into the production install home as `runtimeHome/data`, matching `bin/start.ps1` and `bin/start.sh` bootstrap discovery order.
- Packaged repository `/data` into product server release bundles and documented `SDKWORK_BOOTSTRAP_DATA_DIR=data` for direct binary startup.
- Normalized runtime-tooling platform IDs so both Node-style `win32` and release-matrix `windows` resolve Windows `.exe` binaries and the Windows managed target layout.
- Fixed PowerShell managed-state serialization so empty `AdminAppUrl`, `PortalAppUrl`, or process fingerprint values are written as empty state values instead of aborting startup.
- Hardened Windows installed-runtime smoke startup so background services cannot hold Node pipe handles and block the smoke process.
- Corrected `bin/start-dev.ps1` bootstrap discovery order so source-tree dev runs prefer repository `data/` over stale `bin/data/` remnants from older packaged assets.

## Why This Matters

- Bootstrap data correctness is only commercially useful if the real operator entrypoints can still:
  - discover the right `dev` / `prod` profile
  - initialize writable runtime databases
  - stay idempotent across repeated starts
  - survive stale pid/state files
  - run from both PowerShell and Unix shell wrappers
- The fresh verification round surfaced two real runtime-tooling regressions:
  - stale managed-state parsing could break PowerShell start/stop reuse logic
  - CRLF shell scripts could break installed-runtime Unix/WSL startup even when the business bootstrap data itself was valid
- This round closes both gaps and then re-verifies the full operator command chain.
- A later real install smoke surfaced three additional deployment-only gaps:
  - install plans and product server bundles did not carry bootstrap `/data`
  - Windows release smoke passed `platform=windows`, while install planning only treated `win32` as Windows
  - production `start.ps1` could start a healthy process but fail to persist `state.env` because empty state values were rejected
- These gaps are now covered by regression tests and a real installed-runtime smoke.
- A follow-up real `start-dev.ps1` run exposed a separate source-tree trap:
  - `bin/data/` from an older packaged layout could shadow repository `/data`
  - that stale subset omitted `provider-accounts/`, which broke dev bootstrap validation even though repository `/data` was valid
- The dev entrypoint now prefers repository `/data` and only falls back to `bin/data/` when the repository pack is unavailable.

## Fresh Verification

### Bootstrap and Idempotency

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_discovers_repository_bootstrap_profile_data_pack -- --nocapture`
  - passed: `1 passed; 0 failed`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_discovers_repository_dev_bootstrap_profile_data_pack -- --nocapture`
  - passed: `1 passed; 0 failed`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_applies_bootstrap_profile_data_idempotently -- --nocapture`
  - passed: `1 passed; 0 failed`
- `cargo test -p sdkwork-api-product-runtime product_runtime_bootstraps_repository_default_data_pack -- --nocapture`
  - passed: `1 passed; 0 failed`
- `cargo test -p sdkwork-api-product-runtime product_runtime_bootstraps_repository_dev_identity_seed_data -- --nocapture`
  - passed: `1 passed; 0 failed`

### Managed Runtime Tooling

- `node --test --experimental-test-isolation=none bin/tests/start-dev-windows-backend-warmup.test.mjs bin/tests/router-runtime-tooling.test.mjs`
  - passed: `48 passed; 0 failed; 2 skipped`
  - verified:
    - Windows `start-dev.ps1` dry-run and warm-up plan
    - PowerShell managed state stale-pid handling
    - `start.ps1` dry-run fallback planning
    - `start.sh` WSL/unix dry-run fallback planning
    - installed unix runtime `start.sh` / `stop.sh` end-to-end smoke
    - build/install/runtime plan rendering and bind preflight contracts
- `node --test --experimental-test-isolation=none bin/tests/router-runtime-tooling.test.mjs`
  - passed after follow-up fixes: `48 passed; 0 failed; 2 skipped`
  - verified:
    - install plan packages `data/`
    - normalized `windows` platform IDs still use `.exe` binaries
    - PowerShell state writer accepts empty URL/fingerprint values
    - `start-dev.ps1` prefers repository `/data` before stale `bin/data/`

### Release Packaging and Smoke

- `node --test scripts/release-flow-contract.test.mjs`
  - passed: `9 passed; 0 failed`
  - verified product server bootstrap data roots are exported through release packaging contracts.
- `node --test scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs`
  - passed: `2 passed; 0 failed`
  - verified Windows smoke plans launch `start.ps1` with ignored stdio to avoid background-process pipe hangs.
- `node --check scripts/release/run-windows-installed-runtime-smoke.mjs`
  - passed syntax validation.
- Real Windows release service build with short target dir:
  - `CARGO_TARGET_DIR=C:\sdkrt`
  - `cargo build --release --target x86_64-pc-windows-msvc -p router-product-service -j 1`
  - passed after moving the target dir away from the long repository path.
  - `cargo build --release --target x86_64-pc-windows-msvc -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service -j 1`
  - passed.
- Real external install home verification:
  - `powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Home C:\sdkwork-runtime-manual -Force`
  - passed and installed to `C:/sdkwork-runtime-manual`.
  - verified packaged data files:
    - `C:\sdkwork-runtime-manual\data\channels\default.json`
    - `C:\sdkwork-runtime-manual\data\providers\default.json`
    - `C:\sdkwork-runtime-manual\data\routing\default.json`
- Real external installed runtime startup:
  - `powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\sdkwork-runtime-manual\bin\start.ps1 -Home C:\sdkwork-runtime-manual -WaitSeconds 120 -Bind 127.0.0.1:54803 -GatewayBind 127.0.0.1:54800 -AdminBind 127.0.0.1:54801 -PortalBind 127.0.0.1:54802`
  - passed with exit code `0`.
  - `GET http://127.0.0.1:54803/api/v1/health` returned `ok`.
  - `GET http://127.0.0.1:54803/api/admin/health` returned `ok`.
  - `GET http://127.0.0.1:54803/api/portal/health` returned `ok`.
  - `C:\sdkwork-runtime-manual\var\run\router-product-service.state.env` was written with the expected process id, binds, and `production release` mode.
  - `powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\sdkwork-runtime-manual\bin\stop.ps1 -Home C:\sdkwork-runtime-manual -WaitSeconds 120` passed and removed pid/state files.
- Automated Windows installed-runtime smoke:
  - `CARGO_TARGET_DIR=C:\sdkrt node scripts/release/run-windows-installed-runtime-smoke.mjs --platform windows --arch x64 --target x86_64-pc-windows-msvc --runtime-home C:\sdkwork-runtime-smoke4 --evidence-path artifacts\release-governance\windows-installed-runtime-smoke-windows-x64.json`
  - passed with `ok: true`.
  - verified install, packaged bootstrap data checks, start, health, evidence generation, and stop.
- Real source-tree dev startup after bootstrap precedence fix:
  - `powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -WaitSeconds 300 -GatewayBind 127.0.0.1:59980 -AdminBind 127.0.0.1:59981 -PortalBind 127.0.0.1:59982 -WebBind 127.0.0.1:59983`
  - passed and printed the unified preview access summary.
  - `GET http://127.0.0.1:59980/health` returned `200`.
  - `GET http://127.0.0.1:59981/admin/health` returned `200`.
  - `GET http://127.0.0.1:59982/portal/health` returned `200`.
  - `GET http://127.0.0.1:59983/` returned `200`.
  - `powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\bin\stop-dev.ps1 -WaitSeconds 180` passed and removed active managed processes.

### Operator Dry-Run Commands

- `powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 -DryRun`
  - printed the expected release build plan for binaries, admin, portal, console, docs, and native package output
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -DryRun`
  - printed the expected install plan for binaries, packaged `data/`, sites, `config/router.env`, and native service descriptors
- `SDKWORK_ROUTER_DEV_HOME=<temp>; .\bin\start-dev.ps1 -DryRun`
  - printed the expected dev bootstrap plan
  - confirmed defaults:
    - profile: `dev`
    - db: `<temp>/data/sdkwork-api-router-dev.db`
    - binds: `9980` / `9981` / `9982` / `9983`
- `.\bin\start.ps1 -DryRun -Home <temp>`
  - printed the expected release bootstrap plan
  - confirmed defaults:
    - profile: `prod`
    - db: `<temp>/var/data/sdkwork-api-router.db`
    - binds: `9980` / `9981` / `9982` / `9983`

## Operator Command Matrix

### Windows Dev Verification

```powershell
cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_discovers_repository_dev_bootstrap_profile_data_pack -- --nocapture
cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_applies_bootstrap_profile_data_idempotently -- --nocapture
cargo test -p sdkwork-api-product-runtime product_runtime_bootstraps_repository_dev_identity_seed_data -- --nocapture
node --test --experimental-test-isolation=none bin/tests/start-dev-windows-backend-warmup.test.mjs bin/tests/router-runtime-tooling.test.mjs
$env:SDKWORK_ROUTER_DEV_HOME = "$env:TEMP\\sdkwork-router-dev-verify"
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -DryRun
```

### Windows Release Verification

```powershell
cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_discovers_repository_bootstrap_profile_data_pack -- --nocapture
cargo test -p sdkwork-api-product-runtime product_runtime_bootstraps_repository_default_data_pack -- --nocapture
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 -DryRun
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -DryRun
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1 -DryRun -Home "$env:TEMP\\sdkwork-router-install-verify"
node --test scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs
```

### Windows Real Dev Start

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1
```

Optional overrides:

```powershell
$env:SDKWORK_ROUTER_DEV_HOME = "D:\\runtime\\sdkwork-dev"
$env:SDKWORK_BOOTSTRAP_DATA_DIR = "D:\\your-bootstrap-data"
$env:SDKWORK_BOOTSTRAP_PROFILE = "dev"
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -Browser
```

### Windows Real Release Flow

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1
```

Optional custom install home:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Home "D:\\deploy\\sdkwork-api-router\\current"
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1 -Home "D:\\deploy\\sdkwork-api-router\\current"
```

If Windows release builds fail inside a long workspace path during native C/CMake dependencies, use a short target directory:

```powershell
$env:CARGO_TARGET_DIR = "C:\\sdkrt"
$env:CMAKE_GENERATOR = "Visual Studio 17 2022"
$env:HOST_CMAKE_GENERATOR = "Visual Studio 17 2022"
$env:CARGO_BUILD_JOBS = "1"
cargo build --release --target x86_64-pc-windows-msvc -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service -p router-product-service -j 1
```

Automated installed-runtime smoke:

```powershell
$env:CARGO_TARGET_DIR = "C:\\sdkrt"
node scripts/release/run-windows-installed-runtime-smoke.mjs --platform windows --arch x64 --target x86_64-pc-windows-msvc --runtime-home C:\\sdkwork-runtime-smoke --evidence-path artifacts\\release-governance\\windows-installed-runtime-smoke-windows-x64.json
```

### Linux or macOS Release Flow

```bash
cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_discovers_repository_bootstrap_profile_data_pack -- --nocapture
cargo test -p sdkwork-api-product-runtime product_runtime_bootstraps_repository_default_data_pack -- --nocapture
./bin/build.sh --dry-run
./bin/install.sh --dry-run
./bin/start.sh --dry-run
./bin/build.sh
./bin/install.sh
./bin/start.sh
```

## Acceptance Checklist

- `dev` startup resolves `SDKWORK_BOOTSTRAP_PROFILE=dev`
- `prod` startup resolves `SDKWORK_BOOTSTRAP_PROFILE=prod`
- bootstrap discovery succeeds when using repository `/data`
- repeated bootstrap does not create dirty duplicate seed state
- managed start scripts render valid dry-run plans before touching runtime state
- build/install scripts render valid release/install plans before execution
- production install homes contain packaged bootstrap `data/`
- product server bundles contain packaged bootstrap `data/`
- shell and PowerShell wrappers both remain executable in their target environments
- Windows installed-runtime smoke can install, start, validate health, generate evidence, and stop without hanging on inherited background-process pipes

## Notes

- `bin/start-dev.*` is the source-tree managed runtime and does not require `bin/install.*`.
- `bin/start.*` assumes the install home already exists and is the correct production entrypoint.
- Installed production homes now prefer local `data/` for bootstrap, so commercial deployments do not need access to the source repository.
- If you override `SDKWORK_BOOTSTRAP_DATA_DIR`, the directory must contain `profiles/dev.json` or `profiles/prod.json` plus all referenced grouped JSON bundles.
