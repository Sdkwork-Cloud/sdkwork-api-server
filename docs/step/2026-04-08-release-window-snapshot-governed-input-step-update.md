# 2026-04-08 Release Window Snapshot Governed Input Step Update

## Done

- added governed release-window input support to `compute-release-window-snapshot.mjs`
- accepted both artifact-envelope and raw snapshot payloads
- wired governed release-window input through the governance runner fallback path
- expanded release-window and governance-runner tests to cover the new ingress contract

## Verified

- release-window snapshot tests: `5 / 5`
- governance runner tests: `12 / 12`
- release-sync audit tests: `1 / 1`
- default governance truth still blocks `release-window-snapshot` on this host without governed input
- governed `SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON` turns `release-window-snapshot` green without spawning Git

## Blocking Truth

- this slice closes the release-window governed ingress gap, not the host Git execution policy
- the current local host still blocks direct Node -> Git execution
- `release-sync-audit` remains the next Git-blocked lane that still needs an equivalent governed ingress

## Next

1. mirror the governed-input pattern into `release-sync-audit`
2. define hosted production and retention for release-window governed artifacts
3. keep default blocked lanes truthful until real evidence is supplied
