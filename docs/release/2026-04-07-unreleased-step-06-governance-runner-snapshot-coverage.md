# 2026-04-07 Unreleased Step 06 Governance Runner Snapshot Coverage

## Summary

This Step `06` slice expanded the release-governance entry point so release-window snapshot validation is part of the fixed governance sequence.

## Changes

- added a red test proving `run-release-governance-checks.mjs` must include `release-window-snapshot.test.mjs` in its fixed verification plan
- expanded `listReleaseGovernanceCheckPlans()` so the governance runner now executes:
  - `release-sync-audit.test.mjs`
  - `release-workflow.test.mjs`
  - `release-window-snapshot.test.mjs`
  - `verify-release-sync.mjs --format json`
- added `scripts/release/release-window-snapshot-contracts.mjs`
  - lets the governance runner prove snapshot behavior in fallback mode when Node child execution hits `EPERM`
- kept the live release decision unchanged because multi-repository sync truth is still blocked in the current sandbox

## Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-window-snapshot.test.mjs`
- `node scripts/release/run-release-governance-checks.mjs --format json`

Observed on 2026-04-07:

- the runner red test turned green after snapshot coverage was added
- the targeted release-governance suite passed with `14` tests and `0` failures
- the governance runner now reports three passing fallback contract lanes before the still-blocking live sync audit
- shell-verified release window remained:
  - latest tag `release-2026-03-28-8`
  - `16` commits since tag
  - `627` working-tree entries

## Release Decision

- Status: blocked / unpublished
- Reason: release-governance coverage improved, but live multi-repository sync truth still returns `command-exec-blocked`
- Carry-forward rule: this note must be merged into the next verified successful GitHub release window
