# 2026-04-08 Unreleased Windows Startup Runtime Home Recovery

## 1. Iteration Context

- Wave / Step: `support lane / Windows startup recovery`
- Primary mode: `startup-path-hardening`
- Current state classification: `in_progress`

## 2. Top 3 Candidate Actions

1. Reproduce the Windows `start-dev.ps1` startup chain and fix the router entrypoint so the backend warm-up and dry-run planning path have a stable managed runtime home.
2. Stop after the backend warm-up `cargo build` turns green and leave the PowerShell startup file-persistence path untouched.
3. Ignore the startup scripts and continue only on Step 06 backend business tests.

Action `1` was selected because the user-reported failure happened on the real startup path, not inside an isolated crate-only verification lane.

## 3. Actual Changes

- updated `bin/lib/runtime-common.ps1`
  - added `SDKWORK_ROUTER_DEV_HOME` support to `Get-RouterDefaultDevHome`
  - added `ConvertTo-RouterFileText` and `Write-RouterUtf8File` so generated runtime files can be written through a shared UTF-8 helper instead of direct `Set-Content`
  - switched managed state persistence to the shared file-write helper
- updated `bin/start-dev.ps1`
  - routed dry-run plan persistence and pid-file persistence through `Write-RouterUtf8File`
  - kept the managed short-target Windows warm-up flow intact
- updated `bin/start.ps1`
  - routed release-mode plan persistence and pid-file persistence through `Write-RouterUtf8File`
- updated `bin/tests/start-dev-windows-backend-warmup.test.mjs`
  - added structural coverage for the new runtime-home override and file-write helper
  - added a runtime dry-run proof that targets an isolated `SDKWORK_ROUTER_DEV_HOME` when Node can spawn PowerShell in the current environment

## 4. Verification

- `cargo check -p sdkwork-api-interface-admin -j 1`
- `cargo test -p sdkwork-api-interface-admin --test account_billing_routes -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -- --nocapture`
- `cargo build -p admin-api-service -p gateway-service -p portal-api-service -j 1`
- `node --test --experimental-test-isolation=none bin/tests/start-dev-windows-backend-warmup.test.mjs scripts/dev/tests/windows-rust-toolchain-guard.test.mjs`
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -File bin/start-dev.ps1 -DryRun` with `SDKWORK_ROUTER_DEV_HOME` set to a temp runtime home
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -File bin/start.ps1 -DryRun -RuntimeHome <temp>`

## 5. Architecture / Delivery Impact

- the user-facing Windows startup warm-up path now has real proof that the backend service build can complete under the managed short target directory
- PowerShell startup scripts now expose an explicit isolation hook for dev runtime state, which reduces coupling between scripted verification and the repository-owned `artifacts/runtime/dev/<platform>` directory
- runtime control files are now written through one shared helper instead of duplicating raw file writes across start scripts and managed-state helpers

## 6. Risks / Limits

- in this sandbox, direct `Node -> spawnSync('powershell.exe')` still returns `EPERM`, so the runtime subtest is skipped there even though the same dry-run path was proven manually from PowerShell
- direct `powershell.exe -File bin/start-dev.ps1 -DryRun` against the existing repository-owned default runtime home still showed an access-denied plan-file write in this environment; the new `SDKWORK_ROUTER_DEV_HOME` override provides a stable operator and verification escape hatch
- this slice repaired the startup-path tooling and verification surface; it does not claim Step 06 business closure by itself

## 7. Next Entry

1. keep using the managed short-target build path for Windows startup warm-up validation
2. if the default repository-owned dev runtime home continues to reject plan writes outside this sandbox, decide whether to auto-fallback to a user-writable runtime home or keep the explicit `SDKWORK_ROUTER_DEV_HOME` operator override as the supported path
