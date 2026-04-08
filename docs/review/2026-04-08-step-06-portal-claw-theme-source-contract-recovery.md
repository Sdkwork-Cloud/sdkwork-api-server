# 2026-04-08 Step 06 Portal Claw Theme Source Contract Recovery Review

## Scope

This review slice covered the last remaining red proof in the Portal frontend suite after the billing and recharge commercialization work had already turned green.

Execution boundary:

- repair the concrete red Portal proof only
- do not mutate the external sibling repo `apps/claw-studio`
- preserve the commercial Portal theme contract and the Windows startup recovery facts already established in this workspace

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `contract-recovery`
- Previous mode: `portal-commercial-proof-closure`
- Strategy switch: yes

### Candidate Actions

1. Update the Portal parity proof so it enforces the real contract boundary instead of an obsolete claw repo-relative path depth.
   - `Priority Score: 118`
   - highest value because the remaining failure was in a writable test file, not in production code

2. Attempt to edit the external sibling theme file under `apps/claw-studio`.
   - `Priority Score: 41`
   - rejected because the file is outside the writable root in this session

3. Ignore the red proof because the business UI already looked correct locally.
   - `Priority Score: 12`
   - rejected because it would leave the Step 06 verification surface knowingly red

### Chosen Action

Action 1 was selected because the real contract is:

- claw keeps a repo-relative parent-traversal source directive
- Portal keeps app-local source directives and must not copy claw's repo-relative path

That contract is stable. The old exact `../../../../` depth was not.

## Root Cause Summary

### 1. The Remaining Portal Failure Was A Test Contract Drift, Not A Theme Regression

`apps/sdkwork-router-portal/tests/portal-claw-theme-parity.test.mjs` hardcoded this expectation:

- `@source "../../../../";`

But the current claw reference theme now contains:

- `@source "../../../";`

Result:

- the full Portal suite stayed red even though the shared token, scrollbar, and theme-surface contract still matched
- the test was validating an obsolete repository layout detail instead of the intended theme boundary

### 2. The Windows Startup Failure Class Is Separate From The Portal Theme Proof

During reproduction, a direct root `cargo check -p sdkwork-api-interface-http` hit shared-target Windows linker failures:

- `LNK1201`
- `LNK1136`

But the managed startup lane already passed when routed through:

- `CARGO_TARGET_DIR=bin/.sdkwork-target-vs2022`
- `CARGO_BUILD_JOBS=1`

Result:

- the user-facing startup path was not blocked by the remaining Portal theme proof
- the right architectural decision was to keep the startup conclusion on the managed PowerShell lane and fix the Portal test independently

## Implemented Fix

- updated `apps/sdkwork-router-portal/tests/portal-claw-theme-parity.test.mjs`
- introduced `clawWorkspaceSourceDirectivePattern = /@source "(?:\.\.\/){3,}";/`
- replaced the stale exact-depth claw assertion with the structural parent-traversal assertion
- replaced the Portal negative assertion with the same structural pattern so Portal still cannot silently adopt claw's repo-relative source directive

No production CSS changed in this slice.

## Files Touched In This Slice

- `apps/sdkwork-router-portal/tests/portal-claw-theme-parity.test.mjs`
- `docs/step/2026-04-08-portal-claw-theme-source-contract-recovery-step-update.md`
- `docs/review/2026-04-08-step-06-portal-claw-theme-source-contract-recovery.md`
- `docs/release/2026-04-08-unreleased-portal-claw-theme-source-contract-recovery.md`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-claw-theme-parity.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -DryRun`
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "& { . .\bin\lib\runtime-common.ps1; $null = Enable-RouterManagedCargoEnv -RepoRoot (Get-Location).Path; cargo.exe build -p admin-api-service -p gateway-service -p portal-api-service -j $env:CARGO_BUILD_JOBS }"`
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "$env:SDKWORK_ROUTER_DEV_HOME='artifacts/runtime/dev/windows-x64-codex2'; & .\bin\start-dev.ps1 -WaitSeconds 180"`

### Observed Constraint

- plain root `cargo check -p sdkwork-api-interface-http` still reproduces shared-target Windows linker instability in this workspace and is not the same thing as the managed startup lane
- default isolated `node --test` continues to hit `spawn EPERM` in this sandbox for some Node proof files, so the documented `--experimental-test-isolation=none` runner remains the valid frontend verification path here
- this Codex shell does not preserve the background workspace tree after the parent non-interactive PowerShell command returns, so persistent post-return uptime still requires an operator-side interactive shell check even though the managed readiness path reported healthy before teardown

## Current Assessment

### Closed In This Slice

- the last remaining Portal frontend red proof is green again
- the Portal suite is back to `219 / 219` passing in the documented runner mode
- the claw theme parity proof now guards the intended architectural boundary instead of an obsolete sibling layout detail

### Still Open

- direct shared-target cargo verification on Windows remains less reliable than the managed startup lane
- broader Step 06 commercialization and release-truth work still continue beyond this focused recovery slice

## Next Slice Recommendation

1. keep using the managed Windows startup lane as the operator truth source for backend warm-up
2. keep Portal parity proofs structural when they depend on sibling repo layout details that may legitimately move
