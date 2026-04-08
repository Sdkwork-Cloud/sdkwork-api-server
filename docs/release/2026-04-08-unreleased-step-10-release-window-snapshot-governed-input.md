# Unreleased - Step 10 Release Window Snapshot Governed Input

- Date: 2026-04-08
- Type: patch
- Summary:
  - extended `compute-release-window-snapshot.mjs` so release-window facts can now enter through governed inputs instead of only through local Node -> Git execution
  - added governed input contracts for `--snapshot`, `--snapshot-json`, `SDKWORK_RELEASE_WINDOW_SNAPSHOT_PATH`, and `SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON`
  - validated both artifact-envelope input and raw snapshot input, while keeping the default live Git path unchanged when no governed input is supplied
  - updated `run-release-governance-checks.mjs` fallback replay so the top-level governance runner can consume the same governed release-window input on blocked hosts
- Verification:
  - `release-window-snapshot.test.mjs`: `5 / 5`
  - `release-governance-runner.test.mjs`: `12 / 12`
  - `release-sync-audit.test.mjs`: `1 / 1`
- Remaining truth:
  - the current local host still blocks live Node -> Git execution by default
  - without governed input, `release-window-snapshot` still reports `command-exec-blocked`
  - this slice adds an honest governed ingress path; it does not claim the host-policy blocker is solved
