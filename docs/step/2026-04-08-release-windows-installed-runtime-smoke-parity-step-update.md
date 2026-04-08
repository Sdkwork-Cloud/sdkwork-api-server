# 2026-04-08 Release Windows Installed Runtime Smoke Parity Step Update

## Done

- added `run-windows-installed-runtime-smoke.mjs`
- promoted Windows installed-runtime proof into `release.yml`
- persisted Windows smoke evidence as a governed release artifact
- added Windows smoke evidence attestation and attestation-verification subject coverage

## Verified

- Windows smoke script tests: `2 / 2`
- attestation verification tests: `4 / 4`
- workflow tests: `13 / 13`
- governance runner tests: `11 / 11`
- live repo governance summary: `7` pass, `3` block, `0` fail

## Blocking Truth

- this session verified contracts and fallback-safe governance integration, not a hosted Windows `powershell.exe` smoke run
- `release-slo-governance` still blocks on missing live telemetry input
- `release-window-snapshot` and `release-sync-audit` still block on host-level Git child-exec `EPERM`

## Next

1. collect the first hosted Windows smoke evidence artifact from a PowerShell-capable release lane
2. verify the hosted Windows smoke attestation with `gh attestation verify`
3. continue converting host-blocked live release lanes into hosted evidence instead of fallback-only proof
