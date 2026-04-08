# 2026-04-07 Unreleased Step 06 Live Snapshot Governance Lane

## Summary

This Step `06` slice promoted release-window snapshot collection from a test-only governance dependency into a live governance lane with structured blocked output.

## Changes

- added a red test proving the governance runner must execute an actual `compute-release-window-snapshot.mjs --format json` lane
- expanded `run-release-governance-checks.mjs` so the fixed sequence now contains:
  - `release-sync-audit.test.mjs`
  - `release-workflow.test.mjs`
  - `release-window-snapshot.test.mjs`
  - `compute-release-window-snapshot.mjs --format json`
  - `verify-release-sync.mjs --format json`
- updated `compute-release-window-snapshot.mjs`
  - added `collectReleaseWindowSnapshotResult()`
  - added `isGitCommandExecutionBlockedError()`
  - returns structured `command-exec-blocked` JSON instead of throwing a raw stack trace when Git child execution is denied
- updated the governance runner fallback path so the live snapshot lane still produces structured blocked output in the current sandbox

## Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-window-snapshot.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-window-snapshot.test.mjs`
- `node scripts/release/run-release-governance-checks.mjs --format json`
- `node scripts/release/compute-release-window-snapshot.mjs --format json`

Observed on 2026-04-07:

- the targeted release-governance suite passed with `15` tests and `0` failures
- the live snapshot lane now reports structured blocked JSON:
  - `ok: false`
  - `blocked: true`
  - `reason: command-exec-blocked`
- the runner now shows both snapshot-truth and sync-truth blocked by the same Git child-process sandbox limit
- shell-verified release window remained:
  - latest tag `release-2026-03-28-8`
  - `16` commits since tag
  - `629` working-tree entries

## Release Decision

- Status: blocked / unpublished
- Reason: the release-truth gate is now more explicit, but both live snapshot and live sync remain blocked by Git child-process denial in this sandbox
- Carry-forward rule: this note must be merged into the next verified successful GitHub release window
