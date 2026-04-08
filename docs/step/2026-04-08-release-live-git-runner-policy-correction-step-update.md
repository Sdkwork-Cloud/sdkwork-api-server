# 2026-04-08 Release Live Git Runner Policy Correction Step Update

## Done

- removed `cmd.exe` wrapping from Windows live Git governance runners
- expanded blocked detection from `EPERM` to `EPERM|EACCES`
- hardened release-window and release-sync contract coverage for Windows `shell: false`

## Verified

- release-window snapshot tests: `4 / 4`
- release-sync audit tests: `1 / 1`
- attestation verification tests: `4 / 4`
- governance runner tests: `11 / 11`
- host evidence: direct Node spawn of both `git.exe` and the absolute Git path still returns `EPERM`

## Blocking Truth

- this slice removed the wrapper defect but did not make local live Git lanes pass
- `release-window-snapshot` and `release-sync-audit` are still blocked by host-level Node -> Git execution policy
- `release-slo-governance` is still blocked by missing live telemetry input

## Next

1. design a non-Node or artifact-backed ingress for release-window snapshot facts
2. reuse that ingress pattern for release-sync audit
3. keep blocked release lanes truthful until real hosted evidence exists
