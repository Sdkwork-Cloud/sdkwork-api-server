# 2026-04-08 Release Governance Telemetry Snapshot Contract Review

## 1. Scope

- `scripts/release/materialize-release-telemetry-snapshot.mjs`
- `scripts/release/materialize-slo-governance-evidence.mjs`
- `scripts/release/release-workflow-contracts.mjs`
- `.github/workflows/release.yml`
- `scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
- `scripts/release/tests/materialize-slo-governance-evidence.test.mjs`
- `scripts/release/tests/release-workflow.test.mjs`

## 2. Finding

### P1 live SLO gate still depended on a raw workflow variable

- The previous slice wired a live SLO lane and artifact materializer, but the workflow still injected `SDKWORK_SLO_GOVERNANCE_EVIDENCE_JSON` directly into that materializer.
- Result: the release gate had no repository-owned telemetry snapshot contract, no stable intermediate artifact, and no auditable input boundary between external telemetry and governed SLO evidence.

## 3. Root Cause

- We closed the evaluator and the live lane before closing the producer contract.
- That left the workflow with a test-backed gate but an opaque evidence ingress path.
- The repo already had a working pattern for governed intermediate release truth in the external-dependency and release-window lanes; SLO evidence skipped that pattern.

## 4. Changes

- Added `scripts/release/materialize-release-telemetry-snapshot.mjs`
  - accepts `--snapshot`, `--snapshot-json`, `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH`, and `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON`
  - validates a governed release telemetry snapshot shape against the quantitative SLO baseline target set
  - writes `docs/release/release-telemetry-snapshot-latest.json` by default
  - strips UTF-8 BOM so Windows-authored JSON remains valid
- Updated `scripts/release/materialize-slo-governance-evidence.mjs`
  - keeps direct evidence input for manual/local use
  - can now derive governed SLO evidence from a release telemetry snapshot file or JSON payload
  - expands the missing-input error text so the supported snapshot ingress paths are explicit
- Updated `.github/workflows/release.yml`
  - inserted `Materialize release telemetry snapshot` before `Materialize SLO governance evidence` in native and web release jobs
  - changed the SLO materializer step to consume `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH=docs/release/release-telemetry-snapshot-latest.json`
  - removed direct workflow dependence on raw `SDKWORK_SLO_GOVERNANCE_EVIDENCE_JSON`
- Updated workflow contracts and tests
  - workflow contracts now fail if the snapshot step or snapshot-path wiring is removed
  - snapshot materializer tests now lock helper exports, file input, direct JSON input, missing input, and BOM handling
  - SLO materializer tests now lock snapshot-derived evidence generation

## 5. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
  - `4 tests, 0 fail`
- `node --test --experimental-test-isolation=none scripts/release/tests/materialize-slo-governance-evidence.test.mjs`
  - `5 tests, 0 fail`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
  - `7 tests, 0 fail`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `9 tests, 0 fail`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - `passingIds`: `release-sync-audit-test`, `release-workflow-test`, `release-observability-test`, `release-slo-governance-test`, `release-runtime-tooling-test`, `release-window-snapshot-test`
  - `blockedIds`: `release-slo-governance`, `release-window-snapshot`, `release-sync-audit`
  - `failingIds`: `[]`

## 6. Current Truth

- The workflow now has a governed repository-owned telemetry snapshot boundary before SLO evidence materialization.
- The live SLO lane still blocks by default because `docs/release/slo-governance-latest.json` is absent until the workflow or an operator materializes real telemetry input.
- `release-window-snapshot` and `release-sync-audit` remain blocked in this host because Git child execution still returns `EPERM`.

## 7. Next Step

1. Define who produces `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON` in release CI and what freshness window is acceptable.
2. Decide whether that snapshot comes from a repository-local exporter, a controlled observability job, or an external control plane handoff.
3. Keep the lane blocked until that producer is governed and auditable; do not commit synthetic snapshot or evidence artifacts as repo truth.
