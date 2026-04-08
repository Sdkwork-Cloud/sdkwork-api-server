# 2026-04-08 Unreleased Step 10 Release Governance Observability Gate

## 1. Iteration Context

- Wave / Step: `Step 10 / observability support lane`
- Primary mode: `release-truth hardening`
- Current state classification: `in_progress`

## 2. Top 3 Candidate Actions

1. Add a release-governance observability contract lane so the existing request / routing / runtime / billing evidence chain becomes continuously verifiable.
2. Leave observability as documentation-only truth and continue on unrelated product work.
3. Jump directly to quantitative SLO thresholds before a stable contract gate exists.

Action `1` was selected because `/docs/架构/135` already describes a concrete observability chain, but the release gate did not yet prove that chain stayed intact.

## 3. Actual Changes

- added `scripts/release/observability-contracts.mjs`
  - locks `x-request-id`, `/health`, `/metrics`, tracing bootstrap, routing decision logs, provider health snapshots, runtime-status routes, and billing evidence routes into one contract helper
  - proves that all five service entrypoints still call `init_tracing(...)`
- updated `scripts/release/run-release-governance-checks.mjs`
  - inserted `release-observability-test` into the fixed release-governance sequence
  - added in-process fallback support for the observability lane under child-exec-restricted hosts
- added or updated regression coverage
  - `scripts/release/tests/release-observability-contracts.test.mjs`
  - `scripts/release/tests/release-governance-runner.test.mjs`

## 4. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-observability-contracts.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - the new `release-observability-test` passes
  - the remaining blocked lanes are still live Git-based release-truth checks in this host

## 5. Architecture / Delivery Impact

- observability is now part of executable release truth instead of architecture-only narrative
- restricted hosts can still validate the observability contract lane without reporting false product regressions
- the remaining SLO gap is now narrower and explicit: quantitative thresholds and live burn-rate blocking

## 6. Risks / Limits

- this slice does not claim quantitative SLO governance is complete
- live Git-backed release truth remains blocked in this host by `spawn EPERM`
- Linux/macOS live install smoke and multi-environment observability evidence remain follow-up work

## 7. Next Entry

1. turn observability contract proof into quantitative SLO / burn-rate release thresholds
2. keep live release-truth lanes distinct from sandbox-induced `command-exec-blocked` results
3. continue Step 10 closure without overstating current observability maturity
