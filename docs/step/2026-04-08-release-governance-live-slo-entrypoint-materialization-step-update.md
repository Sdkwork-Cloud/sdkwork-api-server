# 2026-04-08 Release Governance Live SLO Entrypoint Materialization Step Update

## Done

- aligned `run-release-governance-checks.mjs` with the documented live SLO materialization chain
- upgraded missing-input classification from generic `evidence-missing` to specific `telemetry-input-missing`
- proved the single governance entrypoint can pass `release-slo-governance` when governed telemetry export input is present

## Verified

- governance runner tests: `10 / 10`
- telemetry snapshot materializer tests: `4 / 4`
- SLO evidence materializer tests: `5 / 5`
- SLO evaluator tests: `5 / 5`
- default live governance summary: `6` pass, `3` block, `0` fail
- live governance summary with temporary telemetry export input: `7` pass, `2` block, `0` fail

## Blocking Truth

- there is still no real release-time producer for `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`
- telemetry snapshot and SLO evidence are still not persisted as uploaded governance artifacts
- Git child-process policy still blocks `release-window-snapshot` and `release-sync-audit` on this host

## Next

1. persist telemetry snapshot and SLO evidence as dedicated release-governance artifacts
2. wire a real release-time telemetry export producer or control-plane handoff
3. continue closing Git-policy-blocked live lanes on a permitted host
