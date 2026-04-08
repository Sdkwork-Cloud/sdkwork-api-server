# 2026-04-08 Release Governance Governed Evidence Artifacts Review

## 1. Scope

- `.github/workflows/release.yml`
- `scripts/release/release-workflow-contracts.mjs`
- `scripts/release/tests/release-workflow.test.mjs`

## 2. Finding

### P1 governed telemetry and SLO evidence were generated but not persisted as release-governance artifacts

- The workflow already materialized:
  - `docs/release/release-telemetry-snapshot-latest.json`
  - `docs/release/slo-governance-latest.json`
- Those files were consumed by the same job and then discarded.
- Result: the repository had a governed evidence chain, but no retrievable workflow artifact for later audit, triage, or cross-job comparison.

## 3. Root Cause

- The release workflow persisted Unix installed-runtime smoke evidence, but not the governed telemetry snapshot or governed SLO evidence.
- Release workflow contracts only enforced materialization and gate order.
- They did not enforce evidence retention as part of release truth.

## 4. Changes

- Updated `.github/workflows/release.yml`
  - native release jobs now upload `release-governance-telemetry-snapshot-${platform}-${arch}`
  - native release jobs now upload `release-governance-slo-evidence-${platform}-${arch}`
  - web release job now uploads `release-governance-telemetry-snapshot-web`
  - web release job now uploads `release-governance-slo-evidence-web`
  - both uploads run before `Run release governance gate`, so governed evidence is preserved once materialization succeeds
- Updated `scripts/release/release-workflow-contracts.mjs`
  - release truth now requires the two governed evidence uploads in both native and web jobs
  - contract order now requires:
    1. telemetry snapshot materialization
    2. telemetry snapshot artifact upload
    3. SLO evidence materialization
    4. SLO evidence artifact upload
    5. governance gate
- Updated `scripts/release/tests/release-workflow.test.mjs`
  - added red/green proof for native and web governed evidence artifact uploads
  - added rejection tests for workflows that omit telemetry snapshot uploads
  - added rejection tests for workflows that omit SLO evidence uploads

## 5. Industry Alignment

- GitHub Actions documents workflow artifacts as the supported way to keep build and test output after workflow completion and recommends explicit artifact names.
  - Source: https://docs.github.com/en/actions/tutorials/store-and-share-data
- GitHub also positions artifact attestations as the provenance layer for released software and notes that released binaries and packages are the right signing targets.
  - Sources:
    - https://docs.github.com/en/actions/concepts/security/artifact-attestations
    - https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations
- This slice closes the persistence gap and moves the workflow closer to that benchmark, but it does not yet add artifact attestations.

## 6. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
  - `11 / 11`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `10 / 10`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - `passingIds`: `6`
  - `blockedIds`: `3`
  - `failingIds`: `0`
  - unchanged live blockers:
    - `release-slo-governance`: `telemetry-input-missing`
    - `release-window-snapshot`: `command-exec-blocked`
    - `release-sync-audit`: `command-exec-blocked`

## 7. Current Truth

- Governed telemetry snapshot and governed SLO evidence are now persisted as dedicated release-governance artifacts instead of transient job-local files.
- The workflow still does not own a real release-time telemetry export producer.
- The workflow still does not generate artifact attestations for released outputs or governance evidence.

## 8. Next Step

1. Add a real release-time producer or control-plane handoff for `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`.
2. Add provenance/attestation for released artifacts and governed release evidence.
3. Re-run the Git-blocked live lanes on a host that allows Git child-process execution.
