# 2026-04-08 Release Governance Artifact Attestation Step Update

## Done

- added release-workflow permissions required for build provenance
- added attestation steps for governed evidence, Unix smoke evidence, and packaged release assets
- added plan-aware gating for private/internal repositories through `SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED`
- locked the attestation contract into executable workflow tests

## Verified

- release workflow tests: `13 / 13`
- release governance runner tests: `10 / 10`
- default live governance summary: `6` pass, `3` block, `0` fail

## Blocking Truth

- no real hosted-runner attestation was produced in this local session
- there is still no real release-time producer for `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`
- Git child-process policy still blocks `release-window-snapshot` and `release-sync-audit` on this host

## Next

1. capture the first real hosted attestation evidence from the release workflow
2. add operator verification guidance with `gh attestation verify`
3. continue with the telemetry export producer or control-plane handoff
