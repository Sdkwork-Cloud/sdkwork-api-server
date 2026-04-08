# 2026-04-07 Step 06 Release Governance Text Report Fix Review

## Scope

This review slice stayed on Wave `B` / Step `06` and kept the work inside the release-governance lane.

Primary target in this round:

- repair the human-facing `verify-release-sync` text output path so the release gate remains usable outside JSON-only automation

Execution boundary:

- no product behavior changes
- no commit, push, tag, or GitHub release
- keep release truth as the only focus

## Decision Ledger

- Date: `2026-04-07`
- Wave / Step: `B / 06`
- Primary mode: `release-gate-hardening`
- Previous mode: `release-gate-hardening`
- Strategy switch: no

### Top 3 Candidate Actions

1. Repair the broken text-output branch in `scripts/release/verify-release-sync.mjs` and add regression coverage.
   - `Priority Score: 86`
   - direct fix on a real operator-facing failure in the release gate
2. Start wiring the governance runner into `.github/workflows/release.yml` immediately.
   - `Priority Score: 68`
   - valuable, but premature while the governed sibling-repository environment is still not fully materialized in workflow context
3. Limit the round to documentation updates only.
   - `Priority Score: 42`
   - rejected because it would leave a known broken execution path in place

### Chosen Action

Action 1 was selected because it closes a concrete defect in the current release-control surface with low write scope and immediate verification value.

## Implemented Fixes

- updated `scripts/release/verify-release-sync.mjs`
  - replaced the broken text branch that referenced an undefined `reports` variable
  - added `formatReleaseSyncTextReport()` so the text view is deterministic and testable
  - preserved the existing JSON output and blocking exit-code behavior
- updated `scripts/release/tests/release-sync-audit.test.mjs`
  - added contract coverage for the new text formatter
  - verifies both `PASS` and `BLOCK` lines for representative repository states
- updated `docs/release/README.md`
  - documented the stable release-governance entry points
  - documented the sandbox-safe Node test commands
  - documented the blocking reasons that must stop commit / push / release

## Verification Evidence

- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-workflow.test.mjs`
- `node scripts/release/verify-release-sync.mjs --format text`
  - exits non-zero
  - now emits the expected text report instead of throwing on an undefined variable

## Current Assessment

### Closed In This Slice

- the release sync audit is now usable in both machine-readable and operator-readable modes
- the release-governance docs now point to a single stable command chain

### Still Open

- the release gate remains blocked by real repository-sync truth
- GitHub release publication remains unavailable in the current session
- workflow-level enforcement is still a later hardening slice, not yet closed here

## Next Slice Recommendation

1. Keep the release-governance runner as the single verification entry point.
2. Re-check whether the governed sibling repositories can be materialized or audited in a workflow-safe way before wiring the runner into release CI.
3. Do not attempt commit / push / release until the live sync audit turns green in a non-blocked environment.
