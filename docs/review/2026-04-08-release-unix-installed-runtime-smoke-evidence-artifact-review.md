# 2026-04-08 Release Unix Installed Runtime Smoke Evidence Artifact Review

## 1. Scope

- `.github/workflows/release.yml`
- `scripts/release/release-workflow-contracts.mjs`
- `scripts/release/run-unix-installed-runtime-smoke.mjs`
- `scripts/release/tests/release-workflow.test.mjs`
- `scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs`

## 2. Finding

### P0 Unix installed-runtime proof was still non-persistent release output

- The previous slice added a real Unix installed-runtime gate.
- The workflow still treated that gate as transient step output only.
- Result: operators could see a red/green CI step, but could not retrieve a dedicated smoke evidence file from the release lane, especially for post-failure audit.

## 3. Root Cause

- `run-unix-installed-runtime-smoke.mjs` returned JSON to stdout only.
- `.github/workflows/release.yml` had no explicit evidence-path wiring and no dedicated artifact upload step.
- Workflow contracts enforced gate presence and order, but not evidence persistence.

## 4. Changes

- Updated `run-unix-installed-runtime-smoke.mjs`
  - added `--evidence-path`
  - resolved a stable default under `artifacts/release-governance/`
  - exported `createUnixInstalledRuntimeSmokeEvidence`
  - writes JSON evidence on both success and failure
  - includes relative runtime/evidence paths, health URLs, and failure message/log excerpts when available
- Updated `.github/workflows/release.yml`
  - passes `--evidence-path artifacts/release-governance/unix-installed-runtime-smoke-${{ matrix.platform }}-${{ matrix.arch }}.json`
  - uploads `release-governance-unix-installed-runtime-smoke-*` with `if: ${{ always() && matrix.platform != 'windows' }}`
  - keeps governance evidence separate from publishable `release-assets-*`
- Updated release workflow contracts and tests
  - workflow must keep the explicit evidence path
  - workflow must keep the dedicated governance artifact upload
  - helper exports must include evidence creation

## 5. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs`
  - `2 / 2`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
  - `9 / 9`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `9 / 9`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - `passingIds`: `6`
  - `blockedIds`: `3`
  - `failingIds`: `0`

## 6. Current Truth

- Unix native release lanes now persist installed-runtime smoke evidence as a dedicated release-governance artifact.
- Failure cases are better diagnosable because the lane writes evidence before throwing.
- This slice still does not claim a local full built-artifact smoke run in the current sandbox.

## 7. Next Step

1. Add a real Windows installed-runtime evidence lane on a stable PowerShell-capable release host.
2. Convert live SLO evidence from `evidence-missing` to a materialized release-produced artifact.
3. Replace host-blocked Git release-truth lanes with real release-host execution evidence.
