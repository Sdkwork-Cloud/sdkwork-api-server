# 2026-04-08 Release Live Git Runner Policy Review

## 1. Scope

- `scripts/release/compute-release-window-snapshot.mjs`
- `scripts/release/verify-release-sync.mjs`
- `scripts/release/run-release-governance-checks.mjs`
- `scripts/release/verify-release-attestations.mjs`
- `scripts/release/tests/release-window-snapshot.test.mjs`
- `scripts/release/tests/release-sync-audit.test.mjs`
- `scripts/release/tests/release-attestation-verify.test.mjs`
- `scripts/release/tests/release-governance-runner.test.mjs`

## 2. Finding

### P1 Windows live Git governance lanes still carried an avoidable shell-wrapper defect

- `release-window-snapshot` and `release-sync-audit` still blocked on the current host.
- Before this slice, both scripts configured Windows Git execution as `git.exe` with `shell: true`.
- That forced an unnecessary `cmd.exe` wrapper and obscured the true blocker boundary.
- Blocked detection also recognized only `EPERM`, which is too narrow for cross-platform permission-denied hosts where `EACCES` is common.

## 3. Root Cause

- The earlier Windows Git runner policy optimized for command availability, not for release-governance truth.
- `shell: true` on Windows introduced a wrapper process that was not required for `git.exe`.
- Permission-block detection encoded a Windows-biased assumption instead of a cross-platform `permission denied` class.

## 4. Changes

- Corrected Windows Git runner policy
  - `compute-release-window-snapshot.mjs` now exports `resolveGitRunner()` and uses `git.exe` with `shell: false`
  - `verify-release-sync.mjs` now uses `git.exe` with `shell: false`
- Hardened blocked classification
  - `compute-release-window-snapshot.mjs` now treats `EPERM` and `EACCES` as `command-exec-blocked`
  - `verify-release-sync.mjs` now treats `EPERM` and `EACCES` as `command-exec-blocked`
  - `verify-release-attestations.mjs` now classifies `EPERM` and `EACCES` as blocked `gh` execution
  - `run-release-governance-checks.mjs` now triggers fallback for `EPERM` and `EACCES`, and also treats both as blocked in result summaries
- Hardened contracts
  - release-window and release-sync contract helpers now require `shell: false` on Windows Git runners

## 5. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-window-snapshot.test.mjs`
  - `4 / 4`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs`
  - `1 / 1`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-attestation-verify.test.mjs`
  - `4 / 4`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `11 / 11`
- Host evidence
  - `Get-Command git.exe` resolved to `C:\Program Files\Git\cmd\git.exe`
  - `node` direct spawn of `git.exe --version` with `shell: false` still returned `EPERM`
  - `node` direct spawn of absolute-path `C:\Program Files\Git\cmd\git.exe --version` with `shell: false` still returned `EPERM`

## 6. Current Truth

- The `cmd.exe` wrapper defect is closed.
- Permission-denied blocked classification is now cross-platform enough for `EPERM` and `EACCES`.
- The current local host still blocks direct Node child execution of Git even after removing the shell wrapper.
- Therefore this slice improves release-governance correctness and diagnostics, but does not yet turn `release-window-snapshot` or `release-sync-audit` green on this host.

## 7. Remaining Issues

- `release-slo-governance`
  - blocked on missing live telemetry input, not on this Git runner policy
- `release-window-snapshot`
  - now truthfully blocked by host-level Node -> Git execution policy
- `release-sync-audit`
  - now truthfully blocked by host-level Node -> Git execution policy

## 8. Next Step

1. Add a non-Node or artifact-backed live Git fact ingress for release-window snapshotting.
2. Reuse the same ingress pattern for release-sync audit so the multi-repo lane can escape host-local Node child-process policy.
3. Continue keeping blocked lanes truthful instead of degrading them into false PASS states.
