# 2026-04-08 Unreleased Step 10 Release Governance SLO Threshold Baseline

## 1. Iteration Context

- Wave / Step: `Step 10 / quantitative SLO lane`
- Primary mode: `release-truth hardening`
- Current state classification: `in_progress`

## 2. Top 3 Candidate Actions

1. Add a machine-readable SLO baseline plus a release-governance test lane so quantitative thresholds become executable truth.
2. Keep observability at the contract-gate level and postpone all quantitative work.
3. Claim full live burn-rate blocking before a governed evidence artifact exists.

Action `1` was selected because `/docs/架构/135` and `/docs/架构/143` already describe quantitative SLO governance as a required next step, while the repo still lacked a governed threshold source.

## 3. Actual Changes

- added `scripts/release/slo-governance.mjs`
  - defines `14` governed targets across `data-plane`, `control-plane`, and `commercial-plane`
  - encodes `ratio_min`, `ratio_max`, and `value_max` thresholds
  - encodes two burn-rate windows per target
  - evaluates quantitative evidence and exposes a blocked state when evidence is missing
- added `scripts/release/slo-governance-contracts.mjs`
  - locks the three-plane baseline, evidence-source coverage, and burn-rate windows
- added or updated regression coverage
  - `scripts/release/tests/release-slo-governance.test.mjs`
  - `scripts/release/tests/release-governance-runner.test.mjs`
- updated `scripts/release/run-release-governance-checks.mjs`
  - inserted `release-slo-governance-test` into the fixed release-governance sequence
  - added fallback support for the new lane under child-exec-restricted hosts

## 4. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-slo-governance.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - the new `release-slo-governance-test` passes
  - the remaining blocked lanes are still live Git-based release-truth checks in this host
- `node scripts/release/slo-governance.mjs --format json`
  - returns `evidence-missing` until a governed live SLO artifact is materialized

## 5. Architecture / Delivery Impact

- quantitative SLO governance is now encoded as executable repository truth instead of documentation-only guidance
- the release-governance entry point now covers both observability contracts and quantitative SLO baseline logic
- the repo can now distinguish two different maturity levels:
  - baseline and evaluator are present and test-governed
  - live telemetry evidence is still not wired into the release gate

## 6. Risks / Limits

- this slice does not claim that live burn-rate blocking is complete
- this slice does not claim that `docs/release/slo-governance-latest.json` is already produced in CI or release jobs
- live Git-based release lanes remain blocked in this host by `spawn EPERM`

## 7. Next Entry

1. export a governed live SLO evidence artifact
2. add a live SLO release lane once the evidence artifact path is stable
3. keep Step 10 closure evidence-based and avoid overstating SLO maturity
