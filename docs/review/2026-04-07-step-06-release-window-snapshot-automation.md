# 2026-04-07 Step 06 Release Window Snapshot Automation Review

## Scope

This review slice continued Wave `B` / Step `06` on the release-governance lane and focused on release-ledger fact drift.

Primary target in this round:

- replace the manually drifting release-window snapshot with a scriptable baseline

Execution boundary:

- do not change product behavior
- do not change release eligibility rules
- do not commit, push, tag, or publish a GitHub release

## Decision Ledger

- Date: `2026-04-07`
- Wave / Step: `B / 06`
- Primary mode: `release-gate-hardening`
- Previous mode: `release-gate-hardening`
- Strategy switch: no

### Top 3 Candidate Actions

1. Add a script and tests for release-window snapshot facts, then correct the stale ledger count in `CHANGELOG.md`.
   - `Priority Score: 83`
   - closes a real release-record drift source
2. Keep manually editing the snapshot count in `CHANGELOG.md`.
   - `Priority Score: 46`
   - rejected because it guarantees repeat drift
3. Expand into broader release CI changes before stabilizing the decision ledger.
   - `Priority Score: 58`
   - lower leverage than fixing an already stale release fact

### Chosen Action

Action 1 was selected because the release ledger must stay fact-based across repeated execution loops.

## TDD Evidence

Red first:

- added `scripts/release/tests/release-window-snapshot.test.mjs`
- ran:
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-window-snapshot.test.mjs`
- observed expected failure:
  - missing `scripts/release/compute-release-window-snapshot.mjs`

Green after minimal implementation:

- added `scripts/release/compute-release-window-snapshot.mjs`
- re-ran:
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-window-snapshot.test.mjs`
- result:
  - pass

## Implemented Fixes

- added `scripts/release/compute-release-window-snapshot.mjs`
  - computes:
    - latest `release-*` tag
    - commit count since that tag
    - current working-tree entry count
  - supports `--format json`
  - keeps the snapshot logic reusable instead of embedding it in markdown
- added `scripts/release/tests/release-window-snapshot.test.mjs`
  - verifies tag parsing
  - verifies working-tree counting
  - verifies baseline-present and baseline-missing snapshot collection
- updated `docs/release/README.md`
  - documented the new snapshot command
  - documented that the current sandbox still blocks Node child-process Git execution for this class of scripts
- updated `docs/release/CHANGELOG.md`
  - corrected the stale release-window count from `591` to the current shell-verified `618`

## Verification Evidence

- `node --test --experimental-test-isolation=none scripts/release/tests/release-window-snapshot.test.mjs`
- shell-level Git facts used for the current ledger snapshot:
  - latest release tag: `release-2026-03-28-8`
  - commits since tag: `16`
  - current working-tree entries: `618`

### Current Sandbox Constraint

Running:

- `node scripts/release/compute-release-window-snapshot.mjs --format json`

still hits `spawn EPERM` in the current sandbox because Node child-process Git execution is blocked here. That is an environment restriction, not a logic failure in the script. The tested script remains valid for non-blocked local or CI environments.

## Current Assessment

### Closed In This Slice

- the release ledger now has a reusable snapshot script
- the stale workspace-size fact in `CHANGELOG.md` has been corrected

### Still Open

- the new snapshot script is not directly runnable in this sandbox without shell-level Git fallback
- live release truth remains blocked
- no commit / push / GitHub release is authorized

## Next Slice Recommendation

1. Keep updating `/docs/release` from shell-verified facts while Node child-process execution remains blocked here.
2. Use `compute-release-window-snapshot.mjs` in non-blocked environments to prevent future release-ledger drift.
3. Continue Step `06` on the remaining release-truth blockers instead of widening release scope.
