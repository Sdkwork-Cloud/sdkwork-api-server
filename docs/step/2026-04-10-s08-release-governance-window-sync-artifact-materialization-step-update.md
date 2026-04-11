# 2026-04-10 S08 Release Governance Window/Sync Artifact Materialization Step Update

## Scope

- Step: `S08 integrated acceptance, release gate, and continuous iteration`
- Loop target: replace opaque release-governance host-exec blocking with governed latest-artifact truth wherever the current shell can prove it
- Boundaries:
  - `docs/release/release-window-snapshot-latest.json`
  - `docs/release/release-sync-audit-latest.json`
  - `docs/step/110-S08-集成验收-发布门禁与持续迭代-2026-04-10.md`
  - `docs/release/*`
  - `docs/架构/166-*`

## Changes

- release window:
  - materialized `docs/release/release-window-snapshot-latest.json` from current shell git truth
  - proved:
    - `latestReleaseTag = release-2026-03-28-8`
    - `commitsSinceLatestRelease = 22`
    - current `workingTreeEntryCount` at materialization time
- release sync:
  - materialized `docs/release/release-sync-audit-latest.json` from current shell git truth across:
    - `sdkwork-api-router`
    - `sdkwork-core`
    - `sdkwork-ui`
    - `sdkwork-appbase`
    - `sdkwork-im-sdk`
  - converted the lane from `command-exec-blocked` to explicit release `fail` reasons
- documentation:
  - updated `110`, `166`, and release ledger to reflect the new gate structure

## Verification

- shell truth collection:
  - `git tag --list "release-*" --sort=-creatordate | Select-Object -First 5`
  - `git rev-list --count release-2026-03-28-8..HEAD`
  - `git status --short | Measure-Object | Select-Object -ExpandProperty Count`
  - per-repo:
    - `git status --short --branch`
    - `git rev-parse --show-toplevel`
    - `git rev-parse HEAD`
    - `git remote get-url origin`
- artifact materialization:
  - `node scripts/release/materialize-release-window-snapshot.mjs`
  - `node scripts/release/materialize-release-sync-audit.mjs`
- gate replay:
  - `node scripts/release/run-release-governance-checks.mjs --format json`

## Result

- `release-window-snapshot` now passes from governed latest-artifact replay
- `release-sync-audit` now fails with concrete release hygiene and repo-truth reasons instead of remaining an opaque host-exec block
- `release-slo-governance` remains the only still-blocked release-governance lane in the current replay

## Exit

- Step result: `no-go`
- Reason:
  - release truth is more precise and more actionable
  - but release sign-off still cannot pass while sibling repos are dirty, `sdkwork-core` remote/root posture is wrong for governed release, and live telemetry evidence is still absent
