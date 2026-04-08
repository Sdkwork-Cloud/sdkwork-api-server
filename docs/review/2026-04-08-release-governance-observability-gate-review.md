# 2026-04-08 Release Governance Observability Gate Review

## 1. Scope

- `scripts/release/run-release-governance-checks.mjs`
- `scripts/release/observability-contracts.mjs`
- `scripts/release/tests/release-observability-contracts.test.mjs`
- `scripts/release/tests/release-governance-runner.test.mjs`

This slice closes one architecture-to-release gap: observability facts already existed in code and tests, but they were not part of the fixed release-governance truth chain.

## 2. Findings

### P0 release governance had no observability truth lane

- The repo already contains `x-request-id`, `/health`, `/metrics`, HTTP tracing, routing decision logs, provider health snapshots, runtime status views, billing evidence views, and direct regression coverage.
- `run-release-governance-checks.mjs` previously governed `sync / workflow / runtime-tooling / release-window`, but not the observability chain described by `/docs/架构/135`.
- That left a real drift risk: architecture documents could stay correct while release gates stopped proving the observability surface still existed.

### P1 remaining limit

- This patch adds a contract gate, not quantitative SLO governance.
- `run-release-governance-checks.mjs --format json` still exits nonzero in this host, but the remaining blockers are still the live Git-based lanes hitting `command-exec-blocked`, not the new observability lane.

## 3. Changes

- Added `scripts/release/observability-contracts.mjs`
  - checks request-id, metrics, tracing, provider health, commerce recovery, marketing recovery, routing, runtime, and billing evidence contracts
  - proves all five service entrypoints still call `init_tracing(...)`
- Updated `scripts/release/run-release-governance-checks.mjs`
  - inserted `release-observability-test` into the fixed governance sequence
  - added in-process fallback for the observability lane under child-exec-restricted hosts
- Added or updated regression coverage
  - `scripts/release/tests/release-observability-contracts.test.mjs`
  - `scripts/release/tests/release-governance-runner.test.mjs`

## 4. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-observability-contracts.test.mjs`
  - `1 tests, 0 fail`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `7 tests, 0 fail`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - `passingIds`: `release-sync-audit-test`, `release-workflow-test`, `release-observability-test`, `release-runtime-tooling-test`, `release-window-snapshot-test`
  - `blockedIds`: `release-window-snapshot`, `release-sync-audit`
  - `failingIds`: `[]`

## 5. Next Step

1. Promote observability from contract proof to quantitative SLO / burn-rate release thresholds.
2. Keep live Git-based release-truth lanes separate from host-induced `command-exec-blocked` results.
3. Continue Step 10 closure without overstating multi-environment observability maturity.
