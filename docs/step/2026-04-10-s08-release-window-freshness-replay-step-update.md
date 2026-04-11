# 2026-04-10 S08 Release Window Freshness Replay Step Update

## Scope

- Step: `S08 integrated acceptance, release gate, and continuous iteration`
- Loop target: refresh the governed `release-window-snapshot` latest artifact after subsequent doc/release backwrite drifted the stored workspace count
- Boundaries:
  - `docs/release/release-window-snapshot-latest.json`
  - `docs/step/110-S08-集成验收-发布门禁与持续迭代-2026-04-10.md`
  - `docs/release/*`
  - `docs/架构/166-*`

## Changes

- root-cause investigation:
  - compared direct shell `git status --short` counting with the governance replay result
  - confirmed the mismatch came from `compute-release-window-snapshot.mjs` preferring the default latest artifact path when `docs/release/release-window-snapshot-latest.json` already exists
- release window:
  - re-materialized `docs/release/release-window-snapshot-latest.json` from current shell git truth after the latest `S08` doc/release backwrite
  - refreshed:
    - `latestReleaseTag = release-2026-03-28-8`
    - `commitsSinceLatestRelease = 22`
    - `workingTreeEntryCount = 514`
- documentation:
  - updated `110`, `166`, and the release ledger so the commercialization gate reflects the refreshed governed latest-artifact truth instead of the stale `506` snapshot

## Verification

- root-cause and shell truth:
  - `git status --short | Measure-Object | Select-Object -ExpandProperty Count`
  - `git tag --list "release-*" --sort=-creatordate | Select-Object -First 5`
  - `git rev-list --count release-2026-03-28-8..HEAD`
- artifact refresh:
  - `node scripts/release/materialize-release-window-snapshot.mjs --snapshot-json ...`
- gate replay:
  - `node scripts/release/run-release-governance-checks.mjs --format json`

## Result

- `release-window-snapshot` remains `pass`, but now from a refreshed governed latest artifact aligned to the current repo truth
- `release-sync-audit` remains `fail`
- `release-slo-governance` remains `blocked`

## Exit

- Step result: `no-go`
- Reason:
  - this loop removes evidence drift inside the release-window lane
  - but the final release sign-off is still blocked by sibling-repo release hygiene and missing governed live telemetry input
