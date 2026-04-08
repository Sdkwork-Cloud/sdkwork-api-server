# 2026-04-07 Step 06 Release Workflow Ref Contracts Review

## Scope

This review slice continued Wave `B` / Step `06` on the release-governance lane and focused on a workflow-contract blind spot.

Primary target in this round:

- make the workflow contract helper fail when CI stops passing governed repository refs into materialization or governance steps

Execution boundary:

- do not change product behavior
- do not change release runtime logic unless the real workflow is already out of contract
- do not commit, push, tag, or publish a GitHub release

## Decision Ledger

- Date: `2026-04-07`
- Wave / Step: `B / 06`
- Primary mode: `release-gate-hardening`
- Previous mode: `release-gate-hardening`
- Strategy switch: no

### Top 3 Candidate Actions

1. Add a contract test that fails when governed ref env wiring is missing from release workflow steps.
   - `Priority Score: 90`
   - closes a real regression lane in CI truth enforcement
2. Trust the current YAML because the workflow already looks correct today.
   - `Priority Score: 33`
   - rejected because silent future drift would not be caught
3. Expand into more live-sync work before tightening workflow contracts.
   - `Priority Score: 55`
   - lower leverage than closing an existing coverage gap

### Chosen Action

Action 1 was selected because the release workflow must be contract-checked at the same precision level as the governed repository model.

## TDD Evidence

Red first:

- updated `scripts/release/tests/release-workflow.test.mjs`
- added a temporary-repository fixture test that expects `assertReleaseWorkflowContracts()` to reject workflows missing governed ref env wiring
- ran:
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
- observed expected failure:
  - `Missing expected rejection.`

Green after minimal implementation:

- updated `scripts/release/release-workflow-contracts.mjs`
- re-ran:
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
- result:
  - pass

## Implemented Fixes

- `scripts/release/tests/release-workflow.test.mjs`
  - added a temporary fixture builder for workflow-contract regression tests
  - added a contract test for missing governed ref env wiring
- `scripts/release/release-workflow-contracts.mjs`
  - now requires all four governed sibling refs on native/web materialization steps
  - now requires `SDKWORK_API_ROUTER_GIT_REF` plus all four governed sibling refs on native/web governance steps

## Verification Evidence

- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-window-snapshot.test.mjs`
- `node scripts/release/run-release-governance-checks.mjs --format json`
- shell Git facts on 2026-04-07:
  - latest tag `release-2026-03-28-8`
  - commits since tag `16`
  - working-tree entries `624`

Observed result:

- the targeted release-governance suite passed with `13` tests and `0` failures
- the governance runner still blocked on live sync truth with `command-exec-blocked`
- the release gate remained closed

## Current Assessment

### Closed In This Slice

- workflow contract coverage now protects repository-ref env wiring, not just step presence and ordering
- future CI regressions that drop governed ref inputs will fail locally and in contract fallback mode

### Still Open

- live repository sync truth is still blocked in this sandbox
- no commit / push / GitHub release is authorized

## Next Slice Recommendation

1. Keep using the full release-governance suite as the release-truth regression set.
2. Run live sync truth in an environment where Git child-process execution is allowed.
3. Only when live sync truth is green should `commit -> push -> GitHub release` be reconsidered.
