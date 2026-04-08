# 2026-04-08 Release Governance SLO Threshold Baseline Review

## 1. Scope

- `scripts/release/slo-governance.mjs`
- `scripts/release/slo-governance-contracts.mjs`
- `scripts/release/tests/release-slo-governance.test.mjs`
- `scripts/release/run-release-governance-checks.mjs`
- `scripts/release/tests/release-governance-runner.test.mjs`

## 2. Findings

### P1 release governance still lacked a quantitative SLO baseline

- `/docs/架构/135` already defines data-plane, control-plane, and commercial-plane SLO categories plus burn-rate governance expectations.
- The repo already had observability assets, route-level SLO fields, decision logs, runtime status, and billing evidence, but release governance still had no machine-readable threshold baseline.
- That left a real release-risk gap: operators could see metrics, yet CI and release truth still could not prove whether the system stayed inside a governed SLO envelope.

### P1 remaining limit

- This slice adds a quantitative baseline and executable evaluator, not a fully wired live telemetry gate.
- `node scripts/release/slo-governance.mjs --format json` currently returns `blocked=true` with `reason=evidence-missing` because `docs/release/slo-governance-latest.json` is not yet materialized by a live exporter.
- `node scripts/release/run-release-governance-checks.mjs --format json` now includes the SLO test lane and it passes; the only blocked lanes remain the live Git-based release truths in this host.

## 3. Changes

- Added `scripts/release/slo-governance.mjs`
  - defines a machine-readable baseline with `14` governed targets across `data-plane`, `control-plane`, and `commercial-plane`
  - supports `ratio_min`, `ratio_max`, and `value_max` indicators
  - applies fast and slow burn-rate windows (`1h`, `6h`) to every governed target
  - exposes `listSloGovernanceTargets`, `evaluateSloGovernanceEvidence`, and `collectSloGovernanceResult`
- Added `scripts/release/slo-governance-contracts.mjs`
  - proves the baseline exists, stays three-plane complete, cites evidence sources, and keeps burn-rate windows mandatory
- Added `scripts/release/tests/release-slo-governance.test.mjs`
  - covers baseline structure
  - covers passing quantitative evidence
  - covers failing objective and burn-rate evidence
  - covers `evidence-missing`
  - covers collection semantics when a real evidence file exists
- Updated `scripts/release/run-release-governance-checks.mjs`
  - inserted `release-slo-governance-test` into the fixed release-governance sequence
  - added in-process fallback for child-exec-restricted hosts
- Updated `scripts/release/tests/release-governance-runner.test.mjs`
  - locks the new fixed sequence
  - locks aggregation behavior with the new lane
  - locks fallback behavior for the new lane

## 4. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-slo-governance.test.mjs`
  - `5 tests, 0 fail`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `8 tests, 0 fail`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - `passingIds`: `release-sync-audit-test`, `release-workflow-test`, `release-observability-test`, `release-slo-governance-test`, `release-runtime-tooling-test`, `release-window-snapshot-test`
  - `blockedIds`: `release-window-snapshot`, `release-sync-audit`
  - `failingIds`: `[]`
- `node scripts/release/slo-governance.mjs --format json`
  - `blocked=true`
  - `reason=evidence-missing`
  - missing file: `docs/release/slo-governance-latest.json`

## 5. Next Step

1. Materialize a live SLO evidence artifact at `docs/release/slo-governance-latest.json` or an equivalent governed path.
2. Promote the quantitative evaluator from a test-governed lane to a live release-governance lane once the evidence path is stable.
3. Keep platform data policy and SLO governance converging in the same release-truth entry point instead of splitting them across separate, non-blocking narratives.
