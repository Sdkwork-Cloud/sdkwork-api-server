# 2026-04-07 Unreleased Step 06 Release Workflow Ref Contracts

## Summary

This Step `06` slice tightened the release workflow contract layer so CI cannot silently drop repository-ref wiring while still appearing structurally valid.

## Changes

- added a red test that builds a temporary release workflow fixture and proves the contract helper must reject workflows that omit governed repository ref env wiring
- hardened `scripts/release/release-workflow-contracts.mjs` so contract verification now requires:
  - all governed sibling refs on both materialization steps
  - `SDKWORK_API_ROUTER_GIT_REF` plus all governed sibling refs on both governance-gate steps
- kept the runtime workflow unchanged because `.github/workflows/release.yml` already satisfied the stronger contract

## Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-window-snapshot.test.mjs`
- `node scripts/release/run-release-governance-checks.mjs --format json`

Observed on 2026-04-07:

- the workflow test moved from red to green after the contract helper was tightened
- the targeted release-governance suite passed with `13` tests and `0` failures
- live sync truth remained blocked in the current sandbox with `command-exec-blocked`
- shell-verified release window remained:
  - latest tag `release-2026-03-28-8`
  - `16` commits since tag
  - `624` working-tree entries

## Release Decision

- Status: blocked / unpublished
- Reason: the workflow contract is tighter, but live multi-repository sync truth is still not executable in this sandbox
- Carry-forward rule: this note must be merged into the next verified successful GitHub release window
