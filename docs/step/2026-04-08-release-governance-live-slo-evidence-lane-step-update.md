# 2026-04-08 Release Governance Live SLO Evidence Lane Step Update

## Done

- added `scripts/release/materialize-slo-governance-evidence.mjs`
- promoted `release-slo-governance` into the fixed governance sequence
- wired SLO evidence materialization into both release jobs
- hardened workflow contracts and runner/materializer tests
- fixed UTF-8 BOM parsing for Windows evidence files

## Verified

- materializer tests: `4 / 4`
- governance runner tests: `9 / 9`
- workflow tests: `7 / 7`
- live repo governance summary: `6` pass, `3` block, `0` fail

## Blocking Truth

- `release-slo-governance` still blocks by default because `docs/release/slo-governance-latest.json` is not yet produced from real telemetry
- `release-window-snapshot` and `release-sync-audit` still block in this host because Git child execution returns `EPERM`

## Next

1. supply real `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON`
2. define telemetry snapshot producer ownership and freshness policy
3. continue shrinking blocked live lanes without replacing truth with mock artifacts
