# 2026-04-08 Release Unix Installed Runtime Smoke Evidence Artifact Step Update

## Done

- added `--evidence-path` and JSON evidence materialization to `run-unix-installed-runtime-smoke.mjs`
- wired Unix native release lanes to upload smoke evidence as a dedicated governance artifact
- hardened workflow contracts so evidence persistence is mandatory, not optional

## Verified

- Unix smoke script tests: `2 / 2`
- workflow tests: `9 / 9`
- governance runner tests: `9 / 9`
- live governance summary: `6` pass, `3` block, `0` fail

## Blocking Truth

- the current host still blocks live Git child execution for `release-window-snapshot` and `release-sync-audit`
- live SLO governance still blocks on missing evidence
- Windows installed-runtime evidence is still not part of release truth

## Next

1. add Windows installed-runtime release evidence on a stable host
2. materialize live SLO evidence in the release lane
3. keep replacing host-blocked release-truth fallbacks with real release-host proof
