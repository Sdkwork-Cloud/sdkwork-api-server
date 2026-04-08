# Unreleased - Step 10 Release Live Git Runner Policy Correction

- Date: 2026-04-08
- Type: patch
- Summary:
  - corrected Windows live Git governance runners so `compute-release-window-snapshot.mjs` and `verify-release-sync.mjs` now execute `git.exe` with `shell: false` instead of routing through `cmd.exe`
  - broadened blocked execution classification from `EPERM` only to `EPERM|EACCES` across release-window snapshotting, release-sync audit, attestation verification, and the top-level governance runner fallback path
  - hardened release-window and release-sync contracts so future regressions back to `shell: true` on Windows now fail verification immediately
- Verification:
  - `release-window-snapshot.test.mjs`: `4 / 4`
  - `release-sync-audit.test.mjs`: `1 / 1`
  - `release-attestation-verify.test.mjs`: `4 / 4`
  - `release-governance-runner.test.mjs`: `11 / 11`
- Remaining truth:
  - host-level Node -> Git execution is still blocked locally even with direct `git.exe`
  - this slice fixes runner policy and blocked semantics, not the remaining live Git ingress gap
  - release governance still honestly reports blocked live lanes until a non-Node or artifact-backed ingress exists
