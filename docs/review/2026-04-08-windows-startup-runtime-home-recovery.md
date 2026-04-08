# 2026-04-08 Windows Startup Runtime Home Recovery Review

## Scope

This review slice covered the Windows startup entrypoint used by `bin/start-dev.ps1`, including its backend warm-up build, dry-run planning path, and generated runtime control-file persistence.

Execution boundary:

- fix the real startup-path blocker first
- keep the write surface inside runtime helper and startup scripts
- do not overstate this tooling recovery as Step 06 business closure

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Primary mode: `startup-path-hardening`
- Previous mode: `step-06-backend-verification`
- Strategy switch: yes

### Candidate Actions

1. Reproduce the startup chain end to end and repair both the warm-up build path and the PowerShell runtime-home/file-persistence surface.
   - selected because the user-reported failure originated from the startup path itself
2. Stop after the backend warm-up `cargo build` turns green.
   - rejected because the PowerShell entrypoint still needed a stable verification path
3. Ignore startup and continue only on business tests.
   - rejected because it would leave the original operator entrypoint unresolved

## Root Cause Summary

### 1. The Original User Blocker Was the Windows Backend Warm-Up Build

The reported `start-dev.ps1` failure terminated inside the backend warm-up `cargo build` path before the workspace launcher could continue.

Current evidence after this slice:

- `cargo build -p admin-api-service -p gateway-service -p portal-api-service -j 1` is now green when routed through the managed short target directory `bin/.sdkwork-target-vs2022`

### 2. Startup Scripts Still Needed a Stable Runtime-Home Verification Path

Even after the service build recovered, the PowerShell startup scripts still wrote generated plan / pid / state files directly through `Set-Content` and assumed a single repository-owned dev runtime home.

Impact:

- scripted validation had no clean way to isolate runtime state
- repeated verification against the repository-owned runtime tree could be polluted by prior local state

### 3. Node-Based PowerShell Runtime Proof Is Constrained in This Sandbox

The new runtime test exposed that `node:child_process.spawnSync('powershell.exe')` returns `EPERM` in the current sandbox.

Impact:

- the runtime dry-run proof cannot be enforced unconditionally from Node here
- a runtime test needs an explicit spawnability gate instead of converting sandbox process-policy limits into false regressions

## Implemented Fixes

- added `SDKWORK_ROUTER_DEV_HOME` support in `bin/lib/runtime-common.ps1`
- introduced a shared `Write-RouterUtf8File` helper for generated runtime files
- moved managed-state writes onto the shared helper
- switched `bin/start-dev.ps1` and `bin/start.ps1` plan/pid writes onto the shared helper
- added startup contract coverage for:
  - `SDKWORK_ROUTER_DEV_HOME`
  - `Write-RouterUtf8File`
  - PowerShell runtime dry-run proof when Node can legally spawn PowerShell

## Files Touched In This Slice

- `bin/lib/runtime-common.ps1`
- `bin/start-dev.ps1`
- `bin/start.ps1`
- `bin/tests/start-dev-windows-backend-warmup.test.mjs`

## Verification Evidence

### Green

- `cargo check -p sdkwork-api-interface-admin -j 1`
- `cargo test -p sdkwork-api-interface-admin --test account_billing_routes -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -- --nocapture`
- `cargo build -p admin-api-service -p gateway-service -p portal-api-service -j 1`
- `node --test --experimental-test-isolation=none bin/tests/start-dev-windows-backend-warmup.test.mjs scripts/dev/tests/windows-rust-toolchain-guard.test.mjs`
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -File bin/start-dev.ps1 -DryRun` with `SDKWORK_ROUTER_DEV_HOME=<temp>`
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -File bin/start.ps1 -DryRun -RuntimeHome <temp>`

### Observed Constraint

- `Node -> powershell.exe` process spawning still returns `EPERM` in this sandbox, so the runtime dry-run test is intentionally skipped there
- direct `powershell.exe -File bin/start-dev.ps1 -DryRun` against the existing repository-owned default dev runtime home still reproduced an access-denied plan-file write in this environment

## Current Assessment

### Closed In This Slice

- the original Windows backend warm-up service build blocker is green again
- startup scripts now have a controlled isolation hook for dev runtime state
- generated runtime file persistence is centralized behind one helper instead of scattered raw writes

### Still Open

- the repository-owned default dev runtime home still needs a final decision if that access-denied plan write reproduces outside this sandbox
- Step 06 business closure remains separate from this startup tooling recovery

## Next Slice Recommendation

1. keep the explicit `SDKWORK_ROUTER_DEV_HOME` override in the supported Windows operator toolbox
2. decide whether to auto-fallback from the repository-owned dev runtime home when plan writes fail, or preserve the current explicit-override model
