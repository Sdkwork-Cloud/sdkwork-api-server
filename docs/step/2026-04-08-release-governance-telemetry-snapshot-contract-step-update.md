# 2026-04-08 Release Governance Telemetry Snapshot Contract Step Update

## Done

- added `scripts/release/materialize-release-telemetry-snapshot.mjs`
- taught `materialize-slo-governance-evidence.mjs` to derive evidence from a telemetry snapshot
- rewired both release jobs to materialize snapshot first and SLO evidence second
- hardened workflow contracts and snapshot/materializer tests

## Verified

- telemetry snapshot tests: `4 / 4`
- SLO materializer tests: `5 / 5`
- workflow tests: `7 / 7`
- governance runner tests: `9 / 9`
- live repo governance summary: `6` pass, `3` block, `0` fail

## Blocking Truth

- `release-slo-governance` still blocks by default because `docs/release/slo-governance-latest.json` is absent until real telemetry is supplied
- `release-window-snapshot` and `release-sync-audit` still block in this host because Git child execution returns `EPERM`
- the repo now owns the snapshot contract, but it still does not own the upstream snapshot producer

## Next

1. define the release-time producer for `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON`
2. define freshness and provenance policy for that snapshot
3. continue shrinking blocked live lanes without backfilling fake release artifacts
