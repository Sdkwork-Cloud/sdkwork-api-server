# 2026-04-08 Release Governance Governed Evidence Artifacts Step Update

## Done

- persisted governed telemetry snapshot as a dedicated release-governance artifact in native and web release jobs
- persisted governed SLO evidence as a dedicated release-governance artifact in native and web release jobs
- locked both uploads into executable release-workflow contracts and regression tests

## Verified

- release workflow tests: `11 / 11`
- release governance runner tests: `10 / 10`
- default live governance summary: `6` pass, `3` block, `0` fail

## Blocking Truth

- there is still no real release-time producer for `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`
- release evidence is now retained, but not yet attested
- Git child-process policy still blocks `release-window-snapshot` and `release-sync-audit` on this host

## Next

1. add a real telemetry export producer or governed control-plane handoff
2. add provenance or attestation for release assets and governed evidence
3. replay the Git-blocked live lanes on an allowed host
