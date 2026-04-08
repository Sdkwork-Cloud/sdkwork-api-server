# 2026-04-07 Step 06 Release Sync Audit Automation Review

## Scope

This review slice continued Wave `B` / Step `06` on the release-gate track and converted the user-defined repository-sync rules into a machine-readable audit script.

Primary target in this round:

- turn the release gate for `sdkwork-api-router`, `sdkwork-core`, `sdkwork-ui`, `sdkwork-appbase`, and `sdkwork-im-sdk` into executable repository checks

Execution boundary:

- do not change any product behavior
- do not commit, push, or release
- keep the work focused on release truth and dependency synchronization evidence

## Decision Ledger

- Date: `2026-04-07`
- Wave / Step: `B / 06`
- Primary mode: `release-gate-hardening`
- Previous mode: `release-gate-hardening`
- Strategy switch: no

### Top 3 Candidate Actions

1. Add a tested repository-sync audit script that encodes the release gate for the main repo and required dependency repos.
   - `Priority Score: 84`
   - `S1` current-step closure push: `3 x 5 = 15`
   - `S2` Step 06 capability / `8.3` / `8.6` / `95` push: `2 x 5 = 10`
   - `S3` verification and release-gate push: `5 x 4 = 20`
   - `S4` blocker removal value: `4 x 4 = 16`
   - `S5` commercial delivery push: `3 x 3 = 9`
   - `S6` dual-runtime consistency value: `1 x 3 = 3`
   - `S7` immediate verifiability: `5 x 2 = 10`
   - `P1` churn / rework risk: `-1 x 3 = -3`

2. Start a wider Step 06 capability audit across the full staged tree before making the sync gate executable.
   - `Priority Score: 63`
   - lower release-gate leverage than codifying the explicit no-go rule first

3. Attempt to create the mainline snapshot commit despite unresolved remote and dependency sync truth.
   - `Priority Score: 29`
   - rejected because it would directly violate the declared release gate

### Chosen Action

Action 1 was selected because it changes the release gate from a documentation-only rule into a scriptable, testable control with minimal write-surface expansion.

## Implemented Fixes

- added `scripts/release/verify-release-sync.mjs`
  - audits `sdkwork-api-router`, `sdkwork-core`, `sdkwork-ui`, `sdkwork-appbase`, and `sdkwork-im-sdk`
  - encodes expected repository root and expected GitHub remote URL for each repository
  - exports pure helpers for:
    - repository spec enumeration
    - `git status --short --branch` parsing
    - sync-state evaluation
    - overall gate pass / fail
  - emits JSON or text reports
  - exits non-zero when the release sync gate is not satisfied
- added `scripts/release/tests/release-sync-audit.test.mjs`
  - verifies repository specs
  - verifies branch-summary parsing
  - verifies sync blocking for:
    - non-standalone roots
    - dirty working trees
    - remote-unverifiable states
  - verifies Windows / non-Windows git runner selection
  - verifies explicit classification of command-execution blocking

## Important Runtime Constraint

The current sandbox blocks Node child-process git execution with `spawnSync ... EPERM`.

Therefore this script currently behaves in two layers:

- in a normal local environment, it is intended to perform real git audits
- in the current sandbox, it truthfully reports `command-exec-blocked` instead of fabricating repository state from empty command outputs

This is an accuracy improvement over the first draft, which could only infer empty-state failures when Node subprocess execution was denied.

## Files Touched In This Slice

- `scripts/release/verify-release-sync.mjs`
- `scripts/release/tests/release-sync-audit.test.mjs`
- `scripts/release/run-release-governance-checks.mjs`
- `scripts/release/release-sync-audit-contracts.mjs`
- `scripts/release/release-workflow-contracts.mjs`
- `scripts/release/tests/release-governance-runner.test.mjs`

## Verification Evidence

### Green

- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`

### Expected Blocking Output

- `node scripts/release/verify-release-sync.mjs --format json`
  - returns exit code `1`
  - currently reports `command-exec-blocked` for all audited repositories in this sandbox

### Replay In The Current Sandbox

Replayed on `2026-04-07` in the current Codex sandbox:

- `node --test scripts/release/tests/release-sync-audit.test.mjs scripts/release/tests/release-workflow.test.mjs`
  - fails with `spawn EPERM`
  - this is a test-runner isolation limitation, not a contract failure in the release checks
- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs`
  - passes
- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
  - passes
- `node scripts/release/verify-release-sync.mjs --format json`
  - exits `1`
  - reports `command-exec-blocked` for all governed repositories in this sandbox
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - passes
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - exits `1`
  - keeps the release contract lanes green through in-process fallback checks
  - preserves the live release sync audit as the single blocking lane
  - reports `mode: "fallback"` for the contract checks in this sandbox

This replay confirms the documented verification path is correct and that the current no-go release state is still truthfully enforced.

### Earlier Shell-Level Evidence Still Applies

The repository facts previously gathered through shell-level git commands remain the authoritative state for this session:

- `sdkwork-api-router` remote verification still fails with TLS EOF
- `sdkwork-core` still resolves to the parent repo instead of a standalone git root
- `sdkwork-ui`, `sdkwork-appbase`, and `sdkwork-im-sdk` remain dirty

## Current Assessment

### Closed In This Slice

- the repository-sync release gate now exists as executable repository policy
- the gate has focused contract coverage
- sandbox-driven false negatives are now labeled honestly as execution blocking

### Still Open

- direct git execution from Node is blocked in this sandbox
- the actual repository sync gate remains failed in shell-level evidence
- mainline commit / push / GitHub release are still blocked
- Step `06` overall closure remains incomplete

## Maturity Delta

- `stateful standalone` fact maturity: unchanged at `L3`
- `stateless runtime` fact maturity: unchanged at `L2`

This slice improved release governance rather than product capability maturity.

## Next Slice Recommendation

1. Keep using shell-level git evidence as the source of truth in this sandbox.
2. Re-run `verify-release-sync.mjs` in a non-blocked local environment to obtain real repository audit output.
3. Once remote access and dependency repo cleanliness are restored, wire the audit output into the actual commit / push / release decision path.
