# 2026-04-07 Step 06 Release Workflow Governance and Tag Ref Sync Review

## Scope

This review slice continued Wave `B` / Step `06` on the release-governance lane and closed the remaining workflow-contract gap between dependency materialization and CI enforcement.

Primary target in this round:

- make release CI execute the governed dependency model and accept detached tag-style release refs for the main repository

Execution boundary:

- do not change product behavior
- do not change local relative-path SDK development wiring
- do not commit, push, tag, or publish a GitHub release

## Decision Ledger

- Date: `2026-04-07`
- Wave / Step: `B / 06`
- Primary mode: `release-gate-hardening`
- Previous mode: `release-gate-hardening`
- Strategy switch: yes

### Top 3 Candidate Actions

1. Align CI to the full governed repository set and add a real governance step before installs.
   - `Priority Score: 92`
   - closes the last workflow-level truth gap
2. Keep CI materialization limited to the currently audited external dependency surface.
   - `Priority Score: 57`
   - rejected because the governed release-truth model spans all required sibling repositories
3. Stop at local tests and defer tag-ref handling to future release failures.
   - `Priority Score: 41`
   - rejected because tag builds are a first-class release path

### Chosen Action

Action 1 was selected because the release workflow must enforce the same governed repository truth that blocks commit, push, and GitHub release decisions.

## TDD Evidence

Red first:

- updated `scripts/release/tests/release-workflow.test.mjs`
- updated `scripts/release/tests/release-sync-audit.test.mjs`
- ran:
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs`
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
- observed expected failures:
  - materialization spec count was still `1` instead of `4`
  - `resolveReleaseSyncRepositoryRef` and detached-tag handling were missing

Green after minimal implementation:

- updated `scripts/release/materialize-external-deps.mjs`
- updated `scripts/release/verify-release-sync.mjs`
- updated `scripts/release/release-workflow-contracts.mjs`
- updated `.github/workflows/release.yml`
- re-ran the same test commands
- result:
  - pass

## Implemented Fixes

- `scripts/release/materialize-external-deps.mjs`
  - expanded the release materialization spec list to all governed sibling repositories
  - kept local relative-path development dependencies unchanged
- `.github/workflows/release.yml`
  - added governed repository ref env wiring in both release jobs
  - inserted `Run release governance gate` between materialization and install
- `scripts/release/verify-release-sync.mjs`
  - added per-repository expected-ref resolution
  - switched remote verification to `ls-remote origin <expectedRef>`
  - accepted peeled annotated-tag output
  - allowed detached tag-like main-repository refs when local `HEAD` matches the remote release tag object
- `scripts/release/release-workflow-contracts.mjs`
  - locked the workflow contract to the new governance-step ordering and four governed materialization specs

## Verification Evidence

- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs scripts/release/tests/release-window-snapshot.test.mjs`
- `node scripts/release/run-release-governance-checks.mjs --format json`
- shell Git facts on 2026-04-07:
  - latest tag `release-2026-03-28-8`
  - commits since tag `16`
  - working-tree entries `622`

Observed result:

- the full targeted release-governance test set passed with `12` tests and `0` failures
- the governance runner still blocks live sync truth in this sandbox with `command-exec-blocked`
- the release gate remains closed

## Current Assessment

### Closed In This Slice

- CI now enforces governed dependency materialization before installs
- CI now runs the release governance gate in both release jobs
- release sync logic now supports detached tag-style main-repository release refs

### Still Open

- live repository sync truth cannot be proven from this sandbox
- no commit / push / GitHub release is authorized

## Next Slice Recommendation

1. Run the same governance commands in a non-blocked environment with real Git child-process access.
2. Verify all governed sibling repositories are present, clean, and remotely synchronized there.
3. Only after that, merge the full `Unreleased` ledger into the next successful real GitHub release.
