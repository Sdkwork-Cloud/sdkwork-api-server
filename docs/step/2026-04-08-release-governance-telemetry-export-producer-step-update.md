# 2026-04-08 Release Governance Telemetry Export Producer Step Update

## Done

- added export-bundle ingress to the release telemetry snapshot materializer
- derived `gateway/admin/portal` availability targets directly from `sdkwork_http_requests_total`
- rewired release workflow contracts and workflow envs from `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON` to `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`
- preserved direct snapshot override support for manual and local use

## Verified

- snapshot materializer tests: `4 / 4`
- workflow tests: `7 / 7`
- SLO materializer tests: `5 / 5`
- governance runner tests: `9 / 9`
- live repo governance summary: `6` pass, `3` block, `0` fail

## Blocking Truth

- the repo still does not own a real release-time exporter for `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`
- `release-slo-governance` still blocks until live governed artifacts are materialized
- `release-window-snapshot` and `release-sync-audit` still block in this host because Git child execution returns `EPERM`
- non-availability targets still require `supplemental.targets` because current raw metrics are not sufficient for honest direct derivation

## Next

1. add a real export producer job or control-plane handoff for `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`
2. define freshness/provenance policy for that bundle
3. expand direct derivation only after raw telemetry coverage becomes sufficient
