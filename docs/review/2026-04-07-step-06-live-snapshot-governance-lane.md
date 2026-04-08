# 2026-04-07 Step 06 Live Snapshot Governance Lane Review

## Scope

This review slice continued Wave `B` / Step `06` on the release-governance lane and promoted release-window snapshot collection into the live governance path.

Primary target in this round:

- make release-window snapshot truth part of the actual governance runner, not only a supporting test lane

Execution boundary:

- do not change product behavior
- do not weaken release blocking rules
- do not commit, push, tag, or publish a GitHub release

## Decision Ledger

- Date: `2026-04-07`
- Wave / Step: `B / 06`
- Primary mode: `release-gate-hardening`
- Previous mode: `release-gate-hardening`
- Strategy switch: yes

### Top 3 Candidate Actions

1. Promote the snapshot CLI into the live governance sequence and make blocked execution return structured results.
   - `Priority Score: 91`
   - turns release-window truth into an explicit governed lane
2. Keep snapshot only as a test-backed contract and continue relying on external shell commands for facts.
   - `Priority Score: 47`
   - rejected because the single governance entry point would stay incomplete
3. Ignore snapshot blocking because sync audit already blocks release.
   - `Priority Score: 28`
   - rejected because hidden blockers create weak operator truth

### Chosen Action

Action 1 was selected because the release-governance runner should surface every materially relevant live release-truth lane, even when the environment blocks execution.

## TDD Evidence

Red first:

- updated `scripts/release/tests/release-window-snapshot.test.mjs`
- updated `scripts/release/tests/release-governance-runner.test.mjs`
- required:
  - a structured blocked result from `compute-release-window-snapshot.mjs`
  - a new live snapshot lane in the governance runner
- ran:
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-window-snapshot.test.mjs`
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
- observed expected failures:
  - `collectReleaseWindowSnapshotResult` was missing
  - the governance runner fixed plan still omitted the live snapshot lane

Green after minimal implementation:

- updated `scripts/release/compute-release-window-snapshot.mjs`
- updated `scripts/release/run-release-governance-checks.mjs`
- re-ran the same test commands
- result:
  - pass

## Implemented Fixes

- `scripts/release/compute-release-window-snapshot.mjs`
  - added structured blocked-result handling
  - kept real snapshot collection intact for non-blocked environments
- `scripts/release/run-release-governance-checks.mjs`
  - added the live snapshot lane to the fixed governance sequence
  - added fallback support with injected `fallbackSpawnSyncImpl`
- `scripts/release/tests/release-window-snapshot.test.mjs`
  - now verifies structured blocked handling
- `scripts/release/tests/release-governance-runner.test.mjs`
  - now locks the live snapshot lane in the fixed sequence
  - now verifies aggregate result ordering and blocked fallback output for that lane

## Verification Evidence

- `node --test --experimental-test-isolation=none scripts/release/tests/release-window-snapshot.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-window-snapshot.test.mjs`
- `node scripts/release/run-release-governance-checks.mjs --format json`
- `node scripts/release/compute-release-window-snapshot.mjs --format json`
- shell Git facts on 2026-04-07:
  - latest tag `release-2026-03-28-8`
  - commits since tag `16`
  - working-tree entries `629`

Observed result:

- the targeted release-governance suite passed with `15` tests and `0` failures
- the live snapshot lane now reports structured `command-exec-blocked` output instead of crashing
- the governance runner remains blocked because both live snapshot and live sync truth are not executable under this sandbox

## Current Assessment

### Closed In This Slice

- release-window truth is now part of the actual governance entry point
- blocked environments now surface explicit structured snapshot failure instead of opaque crashes

### Still Open

- live repository sync truth is still blocked in this sandbox
- live snapshot truth is also blocked in this sandbox
- no commit / push / GitHub release is authorized

## Next Slice Recommendation

1. Run the widened governance runner in an environment where Git child-process execution is allowed.
2. Use the structured snapshot output to reconcile changelog facts with actual release-window truth there.
3. Only after both live snapshot and live sync lanes are green should release publication be reconsidered.
