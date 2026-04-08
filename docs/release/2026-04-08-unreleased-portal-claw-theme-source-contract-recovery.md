# 2026-04-08 Unreleased Portal Claw Theme Source Contract Recovery

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `portal-proof-closure`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Repair the stale Portal claw-theme parity proof inside the writable workspace and re-run the full Portal suite.
2. Attempt to force the external sibling `apps/claw-studio` theme file back to the previously expected `@source` depth.
3. Ignore the red proof because the Portal theme tokens and scrollbar substrate still render correctly.

Action `1` was selected because the failure was caused by a stale local proof contract, not by a production Portal regression.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/tests/portal-claw-theme-parity.test.mjs`
  - replaced the obsolete exact claw `@source "../../../../";` assertion
  - introduced a structural parent-traversal source-directive contract for the claw reference theme
  - kept the strict Portal-local `@source "./";` and `@source "../packages";` checks
  - kept the negative guard preventing Portal from inheriting claw's repo-relative source path

## 4. Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-claw-theme-parity.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -DryRun`
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "& { . .\bin\lib\runtime-common.ps1; $null = Enable-RouterManagedCargoEnv -RepoRoot (Get-Location).Path; cargo.exe build -p admin-api-service -p gateway-service -p portal-api-service -j $env:CARGO_BUILD_JOBS }"`
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "$env:SDKWORK_ROUTER_DEV_HOME='artifacts/runtime/dev/windows-x64-codex2'; & .\bin\start-dev.ps1 -WaitSeconds 180"`

## 5. Architecture / Delivery Impact

- the Portal theme parity lane now validates the real architecture boundary instead of one frozen sibling repo depth
- the full Portal frontend proof lane is green again in the documented runner mode
- the Windows startup warm-up facts remain aligned with the managed short-target PowerShell lane rather than the less reliable shared-target root cargo path

## 6. Risks / Limits

- the sibling claw theme file is still outside the writable root in this session, so this slice intentionally fixed the local contract instead of mutating external source
- plain isolated `node --test` remains unreliable in this sandbox for some files because of `spawn EPERM`
- the Codex non-interactive shell tears down the managed background workspace after the parent command returns, so long-lived uptime beyond the successful readiness window still needs operator-side confirmation in an interactive PowerShell session

## 7. Next Entry

1. keep Portal parity tests structural when they depend on sibling repo layout
2. continue Step 06 closure and release-truth work on top of the now-green Portal proof surface
