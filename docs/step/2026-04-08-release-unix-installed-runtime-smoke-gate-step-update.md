# 2026-04-08 Release Unix Installed Runtime Smoke Gate Step Update

## Done

- added `run-unix-installed-runtime-smoke.mjs`
- promoted Unix installed-runtime proof into `release.yml`
- hardened workflow contracts so the step is mandatory and correctly ordered

## Verified

- workflow tests: `8 / 8`
- Unix smoke script tests: `2 / 2`
- governance runner tests: `9 / 9`
- live repo governance summary: `6` pass, `3` block, `0` fail

## Blocking Truth

- this session verified the gate contract and script logic, not a local full built-artifact smoke run
- Windows installed-runtime proof is still not part of release truth
- `release-slo-governance`, `release-window-snapshot`, and `release-sync-audit` remain blocked for the same evidence / host-policy reasons as before

## Next

1. add Windows installed-runtime release gating on a stable host
2. persist Unix smoke evidence as a release artifact
3. continue converting blocked live release lanes into real host evidence
