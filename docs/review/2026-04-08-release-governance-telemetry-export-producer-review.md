# 2026-04-08 Release Governance Telemetry Export Producer Review

## 1. Scope

- `scripts/release/materialize-release-telemetry-snapshot.mjs`
- `scripts/release/release-workflow-contracts.mjs`
- `.github/workflows/release.yml`
- `scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
- `scripts/release/tests/release-workflow.test.mjs`

## 2. Finding

### P1 snapshot governance still treated governed snapshot JSON as the upstream ingress

- The previous slice closed `snapshot -> SLO evidence`.
- The release workflow still injected `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON` directly into the snapshot materializer.
- Result: the repository owned the snapshot validator, but not the producer boundary between raw telemetry export and governed snapshot truth.

## 3. Root Cause

- The repo closed the intermediate artifact before closing the export contract that should feed it.
- That kept the release lane auditable only from the snapshot stage onward.
- The observability crate exposes raw Prometheus counters, but the release workflow still skipped the export bundle boundary that should separate raw export from governed snapshot derivation.

## 4. Changes

- Added export-bundle support to `materialize-release-telemetry-snapshot.mjs`
  - new inputs:
    - `--export`
    - `--export-json`
    - `SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH`
    - `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`
  - kept direct snapshot input for manual/local override
- Added governed export helpers
  - `resolveReleaseTelemetryExportInput`
  - `deriveReleaseTelemetrySnapshotFromExport`
- Added direct derivation from raw Prometheus text for:
  - `gateway-availability`
  - `admin-api-availability`
  - `portal-api-availability`
- Burn-rate values for directly derived targets are now computed from the governed SLO baseline objective instead of being required as raw input.
- Kept a mixed evidence model:
  - direct Prometheus derivation where the repo has real raw counters
  - `supplemental.targets` for the remaining targets that cannot yet be honestly derived from current raw metrics
- Rewired `.github/workflows/release.yml` so both release jobs now expect `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON` upstream of the snapshot step.
- Updated workflow contracts and tests so removal of the export env or helper exports now breaks verification.

## 5. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
  - `4 / 4`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
  - `7 / 7`
- `node --test --experimental-test-isolation=none scripts/release/tests/materialize-slo-governance-evidence.test.mjs`
  - `5 / 5`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `9 / 9`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - `passingIds`: `6`
  - `blockedIds`: `3`
  - `failingIds`: `0`

## 6. Current Truth

- The release workflow now owns a clearer chain:
  - `export bundle -> governed snapshot -> governed SLO evidence -> governance gate`
- Direct raw derivation is intentionally limited to the three availability targets backed by `sdkwork_http_requests_total`.
- `routing-simulation-p95-latency` and the remaining non-availability targets still depend on `supplemental.targets`; they are not falsely claimed as raw-Prometheus-derived.
- The live gate still blocks by default because no real release-time export artifact is materialized in this repo by default.

## 7. Next Step

1. Add a real release-time telemetry export producer for `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`.
2. Decide freshness and ownership policy for that export bundle.
3. Expand direct derivation only when raw metrics become sufficient, for example after adding real latency histogram/bucket telemetry.
