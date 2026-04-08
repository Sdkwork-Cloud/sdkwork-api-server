# 2026-04-08 Unreleased - Step 10 Release Governance Live SLO Evidence Lane

## Summary

- added a governed `materialize-slo-governance-evidence` step so release jobs now materialize the live SLO artifact before governance executes
- promoted `release-slo-governance` into the actual release-governance sequence
- fixed Windows BOM parsing so UTF-8 BOM evidence files no longer break the materializer

## Delivered

- `scripts/release/materialize-slo-governance-evidence.mjs`
- `release-slo-governance` lane in `scripts/release/run-release-governance-checks.mjs`
- workflow wiring in `.github/workflows/release.yml`
- contract/test updates in:
  - `scripts/release/release-workflow-contracts.mjs`
  - `scripts/release/tests/materialize-slo-governance-evidence.test.mjs`
  - `scripts/release/tests/release-governance-runner.test.mjs`
  - `scripts/release/tests/release-workflow.test.mjs`

## Verification

- `materialize-slo-governance-evidence.test.mjs`: `4 / 4`
- `release-governance-runner.test.mjs`: `9 / 9`
- `release-workflow.test.mjs`: `7 / 7`
- `run-release-governance-checks.mjs --format json`
  - `6` passing lanes
  - `3` blocked lanes
  - `0` failing lanes

## Current Truth

- live SLO release truth is now wired and executable
- live SLO release readiness is not yet proven in-repo because `docs/release/slo-governance-latest.json` is still absent by default
- this slice has since been tightened by the telemetry snapshot contract, so the workflow upstream handoff is now `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON` and the SLO step consumes the governed snapshot artifact path

## Hold

- do not claim full live release readiness until real evidence export is configured
- do not commit synthetic `docs/release/slo-governance-latest.json` as a substitute for live telemetry truth
