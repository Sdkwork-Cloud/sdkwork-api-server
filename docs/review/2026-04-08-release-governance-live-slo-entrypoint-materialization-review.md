# 2026-04-08 Release Governance Live SLO Entrypoint Materialization Review

## 1. Scope

- `scripts/release/run-release-governance-checks.mjs`
- `scripts/release/tests/release-governance-runner.test.mjs`
- `scripts/release/materialize-release-telemetry-snapshot.mjs`
- `scripts/release/materialize-slo-governance-evidence.mjs`
- `scripts/release/slo-governance.mjs`

## 2. Finding

### P1 release-governance single entrypoint did not replay the live SLO materialization chain

- The workflow already defined:
  - release telemetry export -> governed snapshot
  - governed snapshot -> governed SLO evidence
  - governed SLO evidence -> `release-slo-governance`
- `run-release-governance-checks.mjs` still evaluated `release-slo-governance` directly.
- Result: local governance replay blocked at `evidence-missing` even when the repository already knew how to materialize the missing artifact from governed upstream telemetry input.

## 3. Root Cause

- The live SLO lane fallback called `collectSloGovernanceResult()` only.
- It never attempted:
  - `materializeReleaseTelemetrySnapshot()`
  - `materializeSloGovernanceEvidence()`
- This left the single governance entrypoint weaker than the documented release workflow.

## 4. Changes

- Updated `run-release-governance-checks.mjs`
  - when `release-slo-governance` sees missing evidence, it now:
    1. reuses an existing governed snapshot if present
    2. otherwise tries to materialize a governed telemetry snapshot from release telemetry input
    3. materializes governed SLO evidence
    4. re-runs live SLO evaluation
  - returns `telemetry-input-missing` when no upstream telemetry input exists
  - accepts `env` for spawn and fallback paths so in-process replay can use the same governed inputs as the release workflow
- Updated `release-governance-runner.test.mjs`
  - added red/green proof that missing input is classified as `telemetry-input-missing`
  - added red/green proof that a governed telemetry export makes `release-slo-governance` pass through the single entrypoint
  - verifies temporary governed artifacts are materialized and then cleaned up

## 5. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `10 / 10`
- `node --test --experimental-test-isolation=none scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
  - `4 / 4`
- `node --test --experimental-test-isolation=none scripts/release/tests/materialize-slo-governance-evidence.test.mjs`
  - `5 / 5`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-slo-governance.test.mjs`
  - `5 / 5`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - `passingIds`: `6`
  - `blockedIds`: `3`
  - `failingIds`: `0`
  - `release-slo-governance.reason`: `telemetry-input-missing`
- `node scripts/release/run-release-governance-checks.mjs --format json` with a temporary governed telemetry export input
  - `passingIds`: `7`
  - `blockedIds`: `2`
  - `failingIds`: `0`
  - `release-slo-governance`: passes

## 6. Current Truth

- The single governance entrypoint is now aligned with the documented SLO materialization chain.
- This slice does not claim the repository now owns a real release-time telemetry export producer.
- The local default run still blocks honestly until governed telemetry input is supplied.

## 7. Next Step

1. Add a real release-time producer or control-plane handoff for `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`.
2. Persist telemetry snapshot and SLO governance artifacts as dedicated release-governance artifacts.
3. Continue reducing the remaining Git-policy-blocked live lanes on an allowed host.
