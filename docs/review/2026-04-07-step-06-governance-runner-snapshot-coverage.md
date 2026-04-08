# 2026-04-07 Step 06 Governance Runner Snapshot Coverage Review

## Scope

This review slice continued Wave `B` / Step `06` on the release-governance lane and tightened the fixed governance entry point.

Primary target in this round:

- ensure the governance runner covers release-window snapshot validation, not just workflow and sync-audit contracts

Execution boundary:

- do not change product behavior
- do not change release eligibility rules
- do not commit, push, tag, or publish a GitHub release

## Decision Ledger

- Date: `2026-04-07`
- Wave / Step: `B / 06`
- Primary mode: `release-gate-hardening`
- Previous mode: `release-gate-hardening`
- Strategy switch: no

### Top 3 Candidate Actions

1. Add snapshot validation to the governance runner and provide a fallback contract for blocked environments.
   - `Priority Score: 88`
   - closes a real coverage gap in the single release-governance entry point
2. Leave snapshot validation outside the runner and rely on occasional manual test execution.
   - `Priority Score: 39`
   - rejected because it weakens the fixed release-truth regression set
3. Expand into new live-sync behavior before tightening runner coverage.
   - `Priority Score: 53`
   - lower leverage than fixing an entry-point blind spot

### Chosen Action

Action 1 was selected because `run-release-governance-checks.mjs` is the release-truth gate used by CI and should cover the full supporting release ledger surface that remains executable in this sandbox.

## TDD Evidence

Red first:

- updated `scripts/release/tests/release-governance-runner.test.mjs`
- required:
  - `release-window-snapshot-test` in the fixed plan
  - aggregate results to include that lane
  - fallback success for the snapshot lane under `spawnSync node EPERM`
- ran:
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
- observed expected failures:
  - fixed plan still omitted snapshot coverage
  - snapshot fallback was missing

Green after minimal implementation:

- added `scripts/release/release-window-snapshot-contracts.mjs`
- updated `scripts/release/run-release-governance-checks.mjs`
- re-ran:
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
- result:
  - pass

## Implemented Fixes

- `scripts/release/run-release-governance-checks.mjs`
  - added `release-window-snapshot-test` to the fixed governance sequence
  - added fallback support for that lane using an in-process contract helper
- `scripts/release/release-window-snapshot-contracts.mjs`
  - verifies exported snapshot helpers
  - verifies baseline-present and baseline-missing snapshot collection behavior using stubbed Git responses
- `scripts/release/tests/release-governance-runner.test.mjs`
  - now locks the widened fixed sequence
  - now verifies aggregate reporting with the added snapshot lane
  - now verifies fallback success for the snapshot lane

## Verification Evidence

- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-window-snapshot.test.mjs`
- `node scripts/release/run-release-governance-checks.mjs --format json`
- shell Git facts on 2026-04-07:
  - latest tag `release-2026-03-28-8`
  - commits since tag `16`
  - working-tree entries `627`

Observed result:

- the targeted release-governance suite passed with `14` tests and `0` failures
- the governance runner now reports three passing fallback verification lanes before the live sync block
- the release gate remained closed because live multi-repository sync truth is still blocked

## Current Assessment

### Closed In This Slice

- the single governance entry point now covers release-window snapshot validation
- blocked environments still get executable snapshot validation instead of losing that lane entirely

### Still Open

- live repository sync truth is still blocked in this sandbox
- no commit / push / GitHub release is authorized

## Next Slice Recommendation

1. Keep the widened governance runner as the canonical release-truth entry point.
2. Run the same governance command in an environment where Git child-process execution is allowed.
3. Only after live sync truth is green should release publication be reconsidered.
