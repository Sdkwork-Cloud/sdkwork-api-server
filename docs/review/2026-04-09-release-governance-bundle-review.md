# 2026-04-09 Release Governance Bundle Review

## 1. Scope

- `scripts/release/materialize-release-governance-bundle.mjs`
- `.github/workflows/release.yml`
- `scripts/release/release-workflow-contracts.mjs`
- `scripts/release/tests/materialize-release-governance-bundle.test.mjs`
- `scripts/release/tests/release-workflow.test.mjs`

## 2. Finding

### P1 operator restore still required five separate governance downloads

- architecture docs `162` and `163` already closed:
  - restore back into repository-owned latest paths
  - default latest replay by the real CLI
- operator UX still required downloading five separate governance artifacts before restore.
- this was unnecessary friction for the blocked-host recovery path.

## 3. Root Cause

- release workflow produced five independent governed latest artifacts only:
  - release window snapshot
  - release sync audit
  - telemetry export
  - telemetry snapshot
  - SLO evidence
- the repo had no repository-owned bundle step that grouped those files into one restore-oriented artifact.
- workflow contracts therefore could not enforce a single-download operator handoff.

## 4. Changes

- added `scripts/release/materialize-release-governance-bundle.mjs`
  - reuses the existing restore artifact catalog
  - validates all five inputs through repository validators before bundling
  - writes a single directory at `artifacts/release-governance-bundle/`
  - writes `release-governance-bundle-manifest.json` with restore guidance
- updated `.github/workflows/release.yml`
  - `web-release` now materializes and uploads `release-governance-bundle-web`
- updated `scripts/release/release-workflow-contracts.mjs`
  - bundle step and bundle artifact are now part of the workflow contract
- updated test fixtures
  - governance bundle fixtures now satisfy the real telemetry and SLO validators instead of using under-specified synthetic payloads

## 5. Verification

- targeted:
  - `node --test --experimental-test-isolation=none scripts/release/tests/materialize-release-governance-bundle.test.mjs`
    - `1 / 1`
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
    - `14 / 14`
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
    - `16 / 16`
- aggregate:
  - `node --test --experimental-test-isolation=none <all scripts/release/tests/*.test.mjs>`
    - `76 / 76`
- live governance truth on this host:
  - `node scripts/release/run-release-governance-checks.mjs --format json`
    - `ok=false`
    - `blocked=true`
    - `passingIds=7`
    - `blockedIds=3`
    - `failingIds=[]`

## 6. Current Truth

- operators now have one release-governance download for restore workflows.
- the bundle does not replace the five governed source artifacts.
- the bundle does not change attestation subjects or release gate semantics.
- local default truth remains honestly blocked until real Git or telemetry evidence exists.

## 7. Next Step

1. If operator restore must be executable from a downloaded bundle alone, add a documented bundle extraction example to `docs/release/README.md`.
2. Keep telemetry evidence supply separate; this slice improves transport, not evidence production.
