# 2026-04-07 Unreleased Step 06 Release Window Snapshot Automation

## Summary

This unpublished release note records a release-ledger hardening slice for Wave `B` / Step `06`.

The release window snapshot is no longer a permanently hand-maintained markdown fact. A dedicated script now computes the latest release baseline, commit delta, and working-tree size, and the current `CHANGELOG.md` snapshot has been corrected to the latest shell-verified workspace count.

This is not a published GitHub release and does not unblock release publication by itself.

## Decision Ledger

- Date: `2026-04-07`
- Status: `blocked / unpublished`
- Wave / Step: `B / 06`
- Primary mode: `release-gate-hardening`
- Previous mode: `release-gate-hardening`

### Top 3 Candidate Actions

1. Add a script and tests for release-window snapshots, then correct the stale ledger count.
   - `Priority Score: 83`
2. Keep manually editing the count in markdown.
   - `Priority Score: 46`
3. Expand into broader release CI changes first.
   - `Priority Score: 58`

Action 1 was selected because repeated execution requires a reusable source of truth for release-window bookkeeping.

## Delivered Changes

- added `scripts/release/compute-release-window-snapshot.mjs`
  - computes the latest release tag baseline
  - computes commit delta since that tag
  - computes current working-tree entry count
- added `scripts/release/tests/release-window-snapshot.test.mjs`
  - verifies the snapshot logic under mocked Git results
- updated `docs/release/README.md`
  - documented the new snapshot command
  - documented the current sandbox restriction for Node child-process Git execution
- updated `docs/release/CHANGELOG.md`
  - corrected the pending release window snapshot to:
    - `16` commits since `release-2026-03-28-8`
    - `618` working-tree entries

## Verification Evidence

- `node --test --experimental-test-isolation=none scripts/release/tests/release-window-snapshot.test.mjs`
- shell-level Git snapshot used for the current ledger:
  - latest release tag: `release-2026-03-28-8`
  - commits since tag: `16`
  - working-tree entries: `618`

Current sandbox note:

- `node scripts/release/compute-release-window-snapshot.mjs --format json`
  - remains blocked by `spawn EPERM` here because Node child-process Git execution is denied in this sandbox

## Release Window Impact

This slice improves release-ledger correctness and future reproducibility, but the release window remains unpublished because:

- live repository sync truth is still blocked
- no verified commit / push / GitHub release path exists in the current session

This note remains part of `Unreleased` and must be merged into the next successfully verified GitHub release.
