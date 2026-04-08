# 2026-04-08 Release Telemetry Export Control-Plane Handoff Step Update

## Done

- added `materialize-release-telemetry-export.mjs`
- promoted `docs/release/release-telemetry-export-latest.json` to a first-class governed artifact
- rewired release workflow snapshot derivation to consume `SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH`
- added export artifact upload and attestation coverage
- added attestation verification coverage for `release-telemetry-export`
- enabled default blocked-host replay from the standard export artifact path

## Verified

- producer tests: `3 / 3`
- snapshot tests: `4 / 4`
- governance runner tests: `14 / 14`
- attestation verifier tests: `4 / 4`
- workflow tests: `13 / 13`
- default governance summary: `7` pass / `3` block / `0` fail

## Blocking Truth

- the new export artifact boundary is closed, but no live export input is materialized by default on this host
- `release-slo-governance` still blocks honestly without telemetry input
- `release-window-snapshot` and `release-sync-audit` still block honestly on host Git-exec policy

## Next

1. define the hosted control-plane input source that should feed the export materializer in release execution
2. decide whether `release-window-snapshot` and `release-sync-audit` also need repository-owned latest artifact paths for local replay parity
3. keep the governance summary blocked by default until those upstream artifacts are present
