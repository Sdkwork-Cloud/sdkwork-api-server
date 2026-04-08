# 2026-04-08 Release Governance Attestation Verification Step Update

## Done

- added repository-owned attestation verification script
- added attestation verification fallback contract
- inserted `release-attestation-verify-test` into governance runner
- kept the slice bounded to test/contract truth instead of adding a new live blocked lane

## Verified

- attestation verification tests: `4 / 4`
- release governance runner tests: `11 / 11`
- release workflow tests: `13 / 13`
- default live governance summary: `7` pass, `3` block, `0` fail

## Blocking Truth

- governed snapshot and governed SLO evidence are not materialized in this local workspace by default
- Unix smoke evidence is also absent locally unless a release lane materializes it
- local `gh` verification is blocked on this host for discovered packaged assets

## Next

1. capture a hosted release run with real attestation records
2. add a concise operator playbook for downloading subjects and running local verification
3. continue with the telemetry export producer/control-plane handoff gap
