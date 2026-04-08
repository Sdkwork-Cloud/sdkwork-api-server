# 2026-04-07 Unreleased Step 06 Release Sync Audit Automation

## Summary

This unpublished release note records the automation of the repository-sync release gate.

The repository now contains a dedicated script for auditing whether the main repo and required dependency repos are in a releasable synchronization state before commit / push / release decisions are made.

This is not a published GitHub release and does not unblock release publication by itself.

## Decision Ledger

- Date: `2026-04-07`
- Status: `blocked / unpublished`
- Wave / Step: `B / 06`
- Primary mode: `release-gate-hardening`
- Previous mode: `release-gate-hardening`

### Top 3 Candidate Actions

1. Automate the repository-sync release gate as a script with explicit test coverage.
   - `Priority Score: 84`
2. Expand into broader Step 06 capability auditing before the sync gate is executable.
   - `Priority Score: 63`
3. Attempt a snapshot commit despite unresolved sync truth.
   - `Priority Score: 29`

Action 1 was selected because it turns a documentation rule into a reusable release-control asset.

## Delivered Changes

- added `scripts/release/verify-release-sync.mjs`
  - audits `sdkwork-api-router`, `sdkwork-core`, `sdkwork-ui`, `sdkwork-appbase`, and `sdkwork-im-sdk`
  - supports JSON output and non-zero failure on blocked states
- added `scripts/release/tests/release-sync-audit.test.mjs`
  - contract-tests repository spec coverage, sync-state parsing, and block classification
- added `scripts/release/run-release-governance-checks.mjs`
  - runs the documented release-governance verification chain through one stable entry point
  - preserves the documented verification order even when Node child execution is blocked in this sandbox
  - keeps release contract checks green through in-process fallback validation
  - preserves the live sync audit as a blocking lane instead of masking it
- added `scripts/release/release-sync-audit-contracts.mjs`
  - exposes the repository-sync contract checks as reusable in-process assertions
- added `scripts/release/release-workflow-contracts.mjs`
  - exposes the release-workflow and dependency-materialization contract checks as reusable in-process assertions
- added `scripts/release/tests/release-governance-runner.test.mjs`
  - verifies the fixed verification sequence and blocked-summary aggregation

## Verification Evidence

- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
- `node scripts/release/verify-release-sync.mjs --format json`

In the current sandbox, the audit script returns `command-exec-blocked` because Node child-process git execution is denied with `EPERM`.

That does not replace the shell-level blocker evidence already established for this session. It only makes the blocked execution path explicit and auditable.

Additional replay result from the current execution loop:

- default `node --test` for these release checks fails with `spawn EPERM`
- the documented `--experimental-test-isolation=none` commands pass
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs` passes
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - keeps the two contract lanes green through fallback execution
  - stays blocked only on the live sync audit lane

This means the verification assets are healthy, but they must be run with the documented isolation mode in this sandbox.

## Release Window Impact

This slice improves release governance and future reproducibility, but the release window remains unpublished because:

- remote verification for `sdkwork-api-router` still fails in the current session
- `sdkwork-core`, `sdkwork-ui`, `sdkwork-appbase`, and `sdkwork-im-sdk` still do not satisfy the required synchronized release truth

This note remains part of `Unreleased` and must be merged into the next successfully verified GitHub release.
