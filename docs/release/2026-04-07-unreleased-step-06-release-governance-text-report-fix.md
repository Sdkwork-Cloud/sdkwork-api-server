# 2026-04-07 Unreleased Step 06 Release Governance Text Report Fix

## Summary

This unpublished release note records a repair to the operator-facing `verify-release-sync` output path.

The release gate already had JSON output for automation, but the manual text mode still crashed because the script referenced an undefined variable. This round fixes that defect and documents the stable governance entry points.

This is not a published GitHub release and does not unblock release publication by itself.

## Decision Ledger

- Date: `2026-04-07`
- Status: `blocked / unpublished`
- Wave / Step: `B / 06`
- Primary mode: `release-gate-hardening`
- Previous mode: `release-gate-hardening`

### Top 3 Candidate Actions

1. Fix the broken human-readable text mode in `verify-release-sync` and cover it with tests.
   - `Priority Score: 86`
2. Wire the governance runner directly into release CI immediately.
   - `Priority Score: 68`
3. Update docs only and defer the runtime fix.
   - `Priority Score: 42`

Action 1 was selected because the release gate must stay trustworthy for both operators and scripts.

## Delivered Changes

- updated `scripts/release/verify-release-sync.mjs`
  - fixed the text-output path
  - added `formatReleaseSyncTextReport()` for deterministic operator output
- updated `scripts/release/tests/release-sync-audit.test.mjs`
  - added regression coverage for text-mode output
- updated `docs/release/README.md`
  - documented:
    - `run-release-governance-checks.mjs` as the stable release-governance entry point
    - `verify-release-sync.mjs --format text|json`
    - the required sandbox-safe Node test commands
    - the blocking reason classes that must stop release publication

## Verification Evidence

- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-workflow.test.mjs`
- `node scripts/release/verify-release-sync.mjs --format text`

In the current sandbox, the live audit still returns a blocked result. The difference after this round is that text mode now renders the block report correctly instead of failing before the operator can read it.

## Release Window Impact

This slice improves release-governance reliability and operator usability, but the release window remains unpublished because:

- the governed repositories are still not proven clean and synchronized for release truth
- the current environment still does not justify commit / push / GitHub release

This note remains part of `Unreleased` and must be merged into the next successfully verified GitHub release.
