# 2026-04-08 Release Telemetry Export Control-Plane Handoff Review

## 1. Scope

- `scripts/release/materialize-release-telemetry-export.mjs`
- `scripts/release/materialize-release-telemetry-snapshot.mjs`
- `scripts/release/verify-release-attestations.mjs`
- `scripts/release/release-attestation-verification-contracts.mjs`
- `scripts/release/release-workflow-contracts.mjs`
- `.github/workflows/release.yml`
- `scripts/release/tests/materialize-release-telemetry-export.test.mjs`
- `scripts/release/tests/release-governance-runner.test.mjs`
- `scripts/release/tests/release-attestation-verify.test.mjs`
- `scripts/release/tests/release-workflow.test.mjs`

## 2. Finding

### P1 `release-slo-governance` still had no repository-owned default telemetry export artifact handoff

- Earlier slices closed:
  - telemetry export bundle contract
  - telemetry snapshot artifact
  - SLO evidence artifact
  - workflow evidence upload and attestation
- The missing gap was upstream of the snapshot:
  - no repository-owned producer for `docs/release/release-telemetry-export-latest.json`
  - no workflow step uploading that export as governed evidence
  - no attestation verifier subject for that export
  - no default replay path from export artifact to snapshot on blocked hosts

## 3. Root Cause

- The repo could validate and consume telemetry export input, but it still treated that input as an ambient env handoff instead of a first-class governed artifact.
- That left the release workflow weaker than the intended governance chain:
  - control-plane handoff -> governed telemetry export -> governed telemetry snapshot -> governed SLO evidence -> release gate

## 4. Changes

- Added `scripts/release/materialize-release-telemetry-export.mjs`
  - supports governed direct input:
    - `--export`
    - `--export-json`
    - `SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH`
    - `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`
  - supports control-plane handoff assembly:
    - gateway/admin/portal Prometheus text or path
    - supplemental targets JSON or path
    - generatedAt / source.kind / provenance / freshnessMinutes
  - writes the standard governed artifact:
    - `docs/release/release-telemetry-export-latest.json`
- Updated `materialize-release-telemetry-snapshot.mjs`
  - exported release telemetry export validation for reuse
  - auto-discovers the default export artifact when no explicit snapshot/export input is supplied
- Updated `.github/workflows/release.yml`
  - both release jobs now:
    1. materialize governed telemetry export
    2. upload `release-governance-telemetry-export-*`
    3. derive the governed snapshot from `SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH`
    4. attest export + snapshot + SLO evidence together
- Updated attestation verification
  - `release-telemetry-export` is now a required governed subject
- Updated tests and contracts
  - producer tests for direct bundle input, control-plane assembly, and missing-input rejection
  - governance runner test for default export artifact replay
  - workflow and attestation contract coverage for upload and subject-path closure

## 5. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/materialize-release-telemetry-export.test.mjs`
  - `3 / 3`
- `node --test --experimental-test-isolation=none scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
  - `4 / 4`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `14 / 14`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-attestation-verify.test.mjs`
  - `4 / 4`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
  - `13 / 13`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - default truth: `7` pass / `3` block / `0` fail

## 6. Current Truth

- The repo now owns the governed telemetry export artifact boundary and its workflow/attestation contracts.
- Default local replay still blocks honestly because no live telemetry export input is materialized by default on this host.
- Remaining blocked lanes are:
  - `release-slo-governance`: no default live telemetry export input
  - `release-window-snapshot`: host Git child execution blocked
  - `release-sync-audit`: host Git child execution blocked

## 7. Next Step

1. Define hosted production and retention policy for the telemetry export inputs that feed the new workflow step.
2. Add equivalent default governed artifact replay for `release-window-snapshot` and `release-sync-audit` if repository-owned latest artifacts are required locally.
3. Keep blocked-host truth intact until those upstream artifacts are actually supplied.
