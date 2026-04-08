# 2026-04-08 Unreleased - Step 10 Release Governance Telemetry Snapshot Contract

## Summary

- added a governed release telemetry snapshot artifact so release jobs no longer feed the live SLO gate from a raw evidence JSON variable
- updated SLO materialization to derive evidence from that governed snapshot
- kept live release truth honest: no synthetic snapshot or evidence artifact was committed

## Delivered

- `scripts/release/materialize-release-telemetry-snapshot.mjs`
- snapshot-aware `scripts/release/materialize-slo-governance-evidence.mjs`
- workflow wiring in `.github/workflows/release.yml`
- contract/test updates in:
  - `scripts/release/release-workflow-contracts.mjs`
  - `scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
  - `scripts/release/tests/materialize-slo-governance-evidence.test.mjs`
  - `scripts/release/tests/release-workflow.test.mjs`

## Verification

- `materialize-release-telemetry-snapshot.test.mjs`: `4 / 4`
- `materialize-slo-governance-evidence.test.mjs`: `5 / 5`
- `release-workflow.test.mjs`: `7 / 7`
- `release-governance-runner.test.mjs`: `9 / 9`
- `run-release-governance-checks.mjs --format json`
  - `6` passing lanes
  - `3` blocked lanes
  - `0` failing lanes

## Current Truth

- release workflow truth now has two governed artifacts:
  - `docs/release/release-telemetry-snapshot-latest.json`
  - `docs/release/slo-governance-latest.json`
- the live SLO lane still blocks by default because neither artifact is materialized in this repo by default
- the workflow now expects `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON` as the upstream handoff, not raw SLO evidence JSON

## Hold

- do not claim full live SLO closure until a real snapshot producer exists
- do not commit synthetic snapshot/evidence artifacts as a substitute for release-time telemetry truth
