# 2026-04-08 Release Governance Live SLO Evidence Lane Review

## 1. Scope

- `scripts/release/materialize-slo-governance-evidence.mjs`
- `scripts/release/run-release-governance-checks.mjs`
- `scripts/release/release-workflow-contracts.mjs`
- `.github/workflows/release.yml`
- `scripts/release/tests/materialize-slo-governance-evidence.test.mjs`
- `scripts/release/tests/release-governance-runner.test.mjs`
- `scripts/release/tests/release-workflow.test.mjs`

## 2. Findings

### P1 live SLO gate existed only as a test baseline, not as release truth

- The previous slice added the quantitative baseline and evaluator, but release governance still did not execute a live SLO lane.
- The release workflow also had no governed step that materialized `docs/release/slo-governance-latest.json`.
- Result: CI could prove SLO contract structure, but could not prove release-time evidence availability.

### P1 Windows file interoperability gap

- Command-level verification exposed a real Windows defect: BOM-encoded UTF-8 evidence files failed JSON parsing.
- That would break local operator workflows and any Windows-generated evidence handoff path.

## 3. Changes

- Added `scripts/release/materialize-slo-governance-evidence.mjs`
  - accepts evidence from file path or direct JSON env/input
  - validates governed shape against the release SLO baseline without asserting pass/fail objectives
  - writes the governed artifact to `docs/release/slo-governance-latest.json` or a caller-supplied output path
  - strips UTF-8 BOM so Windows-authored evidence files are accepted
- Updated `scripts/release/run-release-governance-checks.mjs`
  - inserted `release-slo-governance` into the fixed governance sequence
  - added fallback execution so child-process-restricted hosts still evaluate the local live SLO artifact
- Updated `.github/workflows/release.yml`
  - added `Materialize SLO governance evidence` before `Run release governance gate` in both release jobs
  - wired `SDKWORK_SLO_GOVERNANCE_EVIDENCE_JSON` into the materializer step
- Updated workflow contracts and tests
  - release workflow contracts now fail if the governed SLO materialization step or env wiring is removed
  - runner tests now lock live-lane ordering and fallback behavior
  - materializer tests now cover direct JSON, file input, missing input, and UTF-8 BOM files

## 4. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/materialize-slo-governance-evidence.test.mjs`
  - `4 tests, 0 fail`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `9 tests, 0 fail`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
  - `7 tests, 0 fail`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - `passingIds`: `release-sync-audit-test`, `release-workflow-test`, `release-observability-test`, `release-slo-governance-test`, `release-runtime-tooling-test`, `release-window-snapshot-test`
  - `blockedIds`: `release-slo-governance`, `release-window-snapshot`, `release-sync-audit`
  - `failingIds`: `[]`
- `node scripts/release/slo-governance.mjs --format json`
  - `blocked=true`
  - `reason=evidence-missing`
- Temporary artifact proof
  - materializer wrote a temp governed artifact with `baselineId=release-slo-governance-baseline-2026-04-08`
  - evaluator returned `ok=true`, `blocked=false`, `14` passing targets

## 5. Remaining Gaps

- The workflow is now wired for live evidence, but the repository still does not provide real `SDKWORK_SLO_GOVERNANCE_EVIDENCE_JSON` at release time.
- The repo-local live gate therefore remains intentionally blocked until real evidence export is configured.
- `release-window-snapshot` and `release-sync-audit` remain blocked in this host because Git child execution is denied with `EPERM`.

## 6. Next Step

1. Connect `SDKWORK_SLO_GOVERNANCE_EVIDENCE_JSON` to a real release telemetry/export pipeline instead of synthetic/manual payloads.
2. Decide whether the evidence should be generated in CI from telemetry snapshots or injected from an external observability control plane.
3. Keep the lane blocking until that source is governed and auditable; do not backfill fake repo-committed evidence.
