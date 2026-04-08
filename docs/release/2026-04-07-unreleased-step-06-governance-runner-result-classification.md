# 2026-04-07 Unreleased Step 06 Governance Runner Result Classification

## Summary

This Step `06` slice hardened the release-governance runner summary so blocked release-truth lanes are now distinguished from real failing lanes at the top level.

## Changes

- added red tests proving `run-release-governance-checks.mjs` must return explicit top-level classification fields
- updated `run-release-governance-checks.mjs`
  - added `blocked`
  - added `passingIds`
  - added `blockedIds`
  - added `failingIds`
- classified `release-window-snapshot` as blocked when its JSON payload reports `blocked: true` or `reason: command-exec-blocked`
- classified `release-sync-audit` as blocked when its JSON payload contains repository reasons including `command-exec-blocked`
- preserved existing `ok` and per-lane `results` output so current callers remain compatible
- refined the text report so non-passing lanes print `BLOCK` for environment-truth blockers and `FAIL` for real release defects

## Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-window-snapshot.test.mjs`
- `node scripts/release/run-release-governance-checks.mjs --format json`

Observed on 2026-04-07:

- the targeted release-governance suite passed with `16` tests and `0` failures
- the live governance JSON now reports:
  - `ok: false`
  - `blocked: true`
  - `passingIds: [release-sync-audit-test, release-workflow-test, release-window-snapshot-test]`
  - `blockedIds: [release-window-snapshot, release-sync-audit]`
  - `failingIds: []`
- shell-verified release window remained:
  - latest tag `release-2026-03-28-8`
  - `16` commits since tag
  - `631` working-tree entries

## Release Decision

- Status: blocked / unpublished
- Reason: release-truth reporting is clearer, but both live snapshot and live sync remain blocked by Git child-process denial in this sandbox
- Carry-forward rule: this note must be merged into the next verified successful GitHub release window
