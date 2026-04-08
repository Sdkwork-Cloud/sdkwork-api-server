# 2026-04-08 Release Unix Installed Runtime Smoke Gate Review

## 1. Scope

- `.github/workflows/release.yml`
- `scripts/release/release-workflow-contracts.mjs`
- `scripts/release/run-unix-installed-runtime-smoke.mjs`
- `scripts/release/tests/release-workflow.test.mjs`
- `scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs`

## 2. Finding

### P0 native release still lacked a real installed-runtime gate between build and packaging

- `release-governance` already covered repository-level runtime-tooling contracts.
- The release workflow still jumped from native build output directly to `package-release-assets.mjs`.
- Result: Unix release lanes could publish packaged assets without ever proving that real built service binaries could be installed, started, pass health checks, and stop from an installed home.

## 3. Root Cause

- Previous slices closed runtime-tooling contracts and fixture-level smoke.
- The workflow never promoted that proof to an artifact-level release step.
- This left a gap between “repo owns install scripts” and “release lane proved the built install layout actually runs”.

## 4. Changes

- Added `scripts/release/run-unix-installed-runtime-smoke.mjs`
  - parses explicit `--platform`, `--arch`, `--target`
  - rejects Windows lanes honestly
  - materializes a real install home from built release inputs via `createInstallPlan` + `applyInstallPlan`
  - rewrites `router.env` to loopback random ports
  - executes installed `start.sh`
  - probes:
    - `/api/v1/health`
    - `/api/admin/health`
    - `/api/portal/health`
  - executes installed `stop.sh`
  - preserves startup log context on failure
- Rewired `.github/workflows/release.yml`
  - added `Run installed native runtime smoke on Unix`
  - placed it after native desktop builds and before native asset packaging
- Hardened release workflow contracts
  - workflow must keep the Unix smoke step
  - workflow must keep the step in the correct order
  - contract import now requires the smoke helper exports to exist

## 5. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
  - `8 / 8`
- `node --test --experimental-test-isolation=none scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs`
  - `2 / 2`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `9 / 9`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - `passingIds`: `6`
  - `blockedIds`: `3`
  - `failingIds`: `0`

## 6. Current Truth

- Native release lanes now have an explicit Unix installed-runtime gate in the publish workflow.
- This slice closes the workflow gap; it does not claim that the local sandbox executed a full built-artifact smoke run.
- Current blocked governance lanes remain unchanged:
  - `release-slo-governance`: missing live evidence artifact
  - `release-window-snapshot`: Git child exec blocked by host `EPERM`
  - `release-sync-audit`: Git child exec blocked by host `EPERM`

## 7. Next Step

1. Add a Windows installed-runtime gate when a stable PowerShell child-process host is available.
2. Persist Unix smoke output as release evidence instead of only step output.
3. Continue replacing host-blocked fallback lanes with real release-host execution evidence.
