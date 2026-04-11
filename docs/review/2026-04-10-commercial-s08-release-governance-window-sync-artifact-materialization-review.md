# 2026-04-10 Commercial S08 Release Governance Window/Sync Artifact Materialization Review

## Scope

- Architecture reference: `166`
- Step reference: `110`
- Loop focus: turn release-governance `EPERM` ambiguity into governed release truth

## Findings

### P0 - release-sync lane was previously blocked too early to expose the real release hygiene failure set

- before this loop, the current host mostly surfaced `command-exec-blocked`
- impact:
  - the release gate truth was less actionable than the underlying repository state
  - `S08` could not distinguish “host cannot spawn git” from “release repos are objectively not releasable”

### P1 - release-window truth was available locally but not yet materialized into the governed latest artifact path

- local shell git already proved the current tag baseline and commit delta
- impact:
  - one release-governance lane stayed blocked even though the repository already had enough local evidence to close it honestly

## Fix Closure

- materialized `docs/release/release-window-snapshot-latest.json` from current shell git truth
- materialized `docs/release/release-sync-audit-latest.json` from current shell repo/sibling repo truth
- reran the governance gate and confirmed the structure tightened from:
  - `3 blocked`
  to:
  - `1 blocked`
  - `1 fail`
  - `1 pass`

## Verification

- `git tag --list "release-*" --sort=-creatordate | Select-Object -First 5`
- `git rev-list --count release-2026-03-28-8..HEAD`
- `git status --short | Measure-Object | Select-Object -ExpandProperty Count`
- per repo shell git replay for branch/root/head/remote truth
- `node scripts/release/materialize-release-window-snapshot.mjs`
- `node scripts/release/materialize-release-sync-audit.mjs`
- `node scripts/release/run-release-governance-checks.mjs --format json`

## Residual Risks

- `release-slo-governance` still lacks governed live telemetry input
- remote head proof is still not available from the current host
- release-sync remains `fail`, not `pass`, because the underlying repos are not release-clean

## Exit

- Step result: `no-go`
- Reason:
  - ambiguity is reduced, but the release gate truth is still negative
  - the next required work is release hygiene cleanup plus real telemetry evidence, not more artifact-shape work
