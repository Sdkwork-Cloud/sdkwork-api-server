# 2026-04-10 Commercial S08 Release Window Freshness Replay Review

## Scope

- Architecture reference: `166`
- Step reference: `110`
- Loop focus: eliminate stale governed `release-window` evidence after additional `S08` backwrite changed the working-tree truth

## Findings

### P1 - release-window governance replay can silently serve stale workspace-size truth when the latest artifact is not re-materialized after further backwrite

- `compute-release-window-snapshot.mjs` resolves the default latest artifact before any live Git collection
- impact:
  - the gate still reports `pass`, but the recorded `workingTreeEntryCount` can drift away from the current repository truth
  - release evidence becomes less trustworthy even if the pass/fail structure is unchanged

### P1 - the real blocker profile did not change, but the governed evidence package needed freshness repair

- fresh shell counting showed the previously stored `506` workspace count was outdated after the last `S08` doc/release edits
- impact:
  - without refreshing the artifact, the repo would continue carrying a technically stale release-window snapshot into later loops

## Fix Closure

- re-materialized `docs/release/release-window-snapshot-latest.json` from current shell git truth
- reran the governance gate and confirmed the structure is unchanged:
  - `release-window-snapshot`: `pass`
  - `release-sync-audit`: `fail`
  - `release-slo-governance`: `blocked`
- backwrote `110`, `166`, and the release ledger with the freshness-replay conclusion

## Verification

- `git status --short | Measure-Object | Select-Object -ExpandProperty Count`
- `git tag --list "release-*" --sort=-creatordate | Select-Object -First 5`
- `git rev-list --count release-2026-03-28-8..HEAD`
- `node scripts/release/materialize-release-window-snapshot.mjs --snapshot-json ...`
- `node scripts/release/run-release-governance-checks.mjs --format json`

## Residual Risks

- `release-sync-audit` is still negative because the governed repository set is not release-clean
- `release-slo-governance` still lacks governed live telemetry input
- additional doc or release backwrite in later loops will require another release-window re-materialization if the workspace count changes again

## Exit

- Step result: `no-go`
- Reason:
  - release evidence is now fresher
  - release readiness is still blocked by external governance truth, not by `S03/S06/S07` product convergence
