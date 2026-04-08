# Release Live Git Runner Policy Correction

> Date: 2026-04-08
> Goal: remove the avoidable Windows shell-wrapper defect from live Git governance lanes and align blocked execution semantics across Windows, Linux, and macOS.

## 1. Problem

- `release-window-snapshot` and `release-sync-audit` are live governance lanes.
- On Windows they previously used `git.exe` with `shell: true`.
- That forced an unnecessary `cmd.exe` wrapper and blurred the true release-host boundary.
- Blocked execution detection recognized only `EPERM`, which is incomplete for cross-platform permission-denied hosts.

## 2. Policy

- Windows Git commands in release governance must use:
  - executable: `git.exe`
  - process mode: `shell: false`
- Blocked execution detection must treat both:
  - `EPERM`
  - `EACCES`
  as the same governed class: `command-exec-blocked`

## 3. Why This Matters

- `shell: true` adds a wrapper process that is not part of release truth.
- If the wrapper fails first, operators see the wrong execution boundary.
- Cross-platform release governance must classify permission-denied hosts consistently, not only on the Windows-specific error variant.

## 4. Scope Of The Correction

- corrected the Windows Git runner contract in:
  - `compute-release-window-snapshot.mjs`
  - `verify-release-sync.mjs`
- corrected blocked classification in:
  - `compute-release-window-snapshot.mjs`
  - `verify-release-sync.mjs`
  - `verify-release-attestations.mjs`
  - `run-release-governance-checks.mjs`

## 5. Honest Boundary

- This correction removes an avoidable wrapper defect.
- It does not guarantee that the current local host permits Node -> Git execution.
- If direct `git.exe` still returns `EPERM` or `EACCES`, the lane must remain blocked.

## 6. Remaining Closure

- The next architecture step is not another wrapper tweak.
- The next real closure is a governed non-Node or artifact-backed ingress for:
  - release-window snapshot facts
  - release-sync audit facts
- Until that exists, blocked local hosts must stay blocked honestly.
