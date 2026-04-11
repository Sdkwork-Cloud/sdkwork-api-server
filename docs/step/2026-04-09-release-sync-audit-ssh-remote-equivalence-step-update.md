# 2026-04-09 Release Sync Audit SSH Remote Equivalence Step Update

## Done

- fixed release-sync remote comparison so GitHub SSH and HTTPS URLs for the same repo are treated as equivalent
- added a red-first regression in `release-sync-audit.test.mjs`
- re-verified:
  - `release-sync-audit.test.mjs`
  - `materialize-release-sync-audit.test.mjs`
  - `release-governance-runner.test.mjs`

## Verified

- `release-sync-audit.test.mjs`: `3 / 3`
- `materialize-release-sync-audit.test.mjs`: `3 / 3`
- `release-governance-runner.test.mjs`: `16 / 16`
- `run-release-governance-checks.mjs --format json`: `7` pass / `3` block / `0` fail

## Blocking Truth

- `release-slo-governance` still blocks without governed telemetry input
- `release-window-snapshot` and `release-sync-audit` still block on this host's Node child-Git policy
- local shell evidence also shows the current workspace is ahead and dirty, so release-sync would still fail truthful release readiness after host Git execution is restored

## Next

1. continue Step 11/13 on governed telemetry supply and operator-ready latest-artifact materialization
2. only attempt final release-sync closure after the workspace is clean and synced
