# 2026-04-08 Portal Claw Theme Source Contract Recovery Step Update

## Slice Goal

Close the remaining Step 06 Portal theme parity failure by removing the stale hardcoded claw theme source-depth assumption while keeping the intended Portal versus claw-studio source-boundary contract intact.

## Closed In This Slice

- reproduced the remaining red proof in `apps/sdkwork-router-portal/tests/portal-claw-theme-parity.test.mjs`
- confirmed the active Windows startup warm-up path is already green when routed through the managed short target directory `bin/.sdkwork-target-vs2022` with `CARGO_BUILD_JOBS=1`
- replaced the stale claw `@source "../../../../";` assertion with a structural parent-traversal contract
- kept the Portal assertions strict for `@source "./";` and `@source "../packages";`
- kept the negative proof that Portal must not copy the claw repo-relative `@source` directive

## Runtime / Display Truth

### Portal Theme Parity Is Structural Again

- the claw reference theme still has to expose the shared token and scrollbar substrate
- the test now proves that claw keeps a repo-relative parent-traversal `@source` directive without assuming one obsolete directory depth
- the Portal theme still must stay app-local:
  - `@source "./";`
  - `@source "../packages";`
  - no claw-style repo-root relative source path

### Windows Startup Warm-Up Remains Governed By The Managed Target Lane

- a direct root `cargo check -p sdkwork-api-interface-http` can still hit shared-target Windows linker failures such as `LNK1201`
- the user-facing startup path is the managed PowerShell lane, which now proves green with:
  - `CARGO_TARGET_DIR=bin/.sdkwork-target-vs2022`
  - `CARGO_BUILD_JOBS=1`
  - `cargo build -p admin-api-service -p gateway-service -p portal-api-service`

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-claw-theme-parity.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -DryRun`
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "& { . .\bin\lib\runtime-common.ps1; $null = Enable-RouterManagedCargoEnv -RepoRoot (Get-Location).Path; cargo.exe build -p admin-api-service -p gateway-service -p portal-api-service -j $env:CARGO_BUILD_JOBS }"`

## Remaining Follow-Up

1. If claw-studio theme structure moves again, keep the Portal proof focused on contract shape instead of hardcoding the exact sibling repo depth.
2. Keep Windows operator guidance on the managed startup lane instead of ad-hoc shared-target cargo verification.
