# 2026-04-08 Unreleased - Step 10 Release Governance Telemetry Export Producer

## Summary

- changed the release snapshot ingress from governed snapshot JSON to a governed telemetry export bundle
- taught the snapshot materializer to derive the three availability targets directly from raw Prometheus text
- kept the rest of the SLO baseline intact by using `supplemental.targets` instead of deleting targets or faking raw derivation

## Delivered

- export-bundle support in `scripts/release/materialize-release-telemetry-snapshot.mjs`
- workflow contract updates in `scripts/release/release-workflow-contracts.mjs`
- release workflow env update in `.github/workflows/release.yml`
- focused regression updates in:
  - `scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
  - `scripts/release/tests/release-workflow.test.mjs`

## Verification

- `materialize-release-telemetry-snapshot.test.mjs`: `4 / 4`
- `release-workflow.test.mjs`: `7 / 7`
- `materialize-slo-governance-evidence.test.mjs`: `5 / 5`
- `release-governance-runner.test.mjs`: `9 / 9`
- `run-release-governance-checks.mjs --format json`
  - `6` passing lanes
  - `3` blocked lanes
  - `0` failing lanes

## Current Truth

- the workflow now expects `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON` upstream of the snapshot step
- the snapshot materializer still accepts direct snapshot JSON/file input for local override and operator debugging
- only the three availability targets are directly raw-derived today
- the live lane remains blocked until a real export producer materializes release-time telemetry

## Hold

- do not claim raw-Prometheus derivation for latency or business-plane targets yet
- do not commit synthetic `latest` artifacts as release truth
- do not treat the remaining blocked lanes as fixed; they are still blocked by missing live artifacts and host `EPERM` Git execution
