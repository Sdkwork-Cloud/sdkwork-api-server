# Release Governance Default Latest CLI Replay

> Date: 2026-04-08
> Goal: make restored governance latest artifacts first-class inputs for the real CLI entrypoints, not only fallback-only replay paths.

## 1. Problem

- architecture docs `161` and `162` already defined:
  - repository-owned latest artifacts
  - blocked-host restore into standard latest paths
- actual CLI child scripts for:
  - `release-window-snapshot`
  - `release-sync-audit`
  still skipped those paths unless explicit env/CLI input was passed.

## 2. Required Input Order

- `compute-release-window-snapshot.mjs`
  1. explicit `--snapshot`
  2. explicit `--snapshot-json`
  3. `SDKWORK_RELEASE_WINDOW_SNAPSHOT_PATH`
  4. `SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON`
  5. default `docs/release/release-window-snapshot-latest.json`
  6. live Git collection
- `verify-release-sync.mjs`
  1. explicit `--audit`
  2. explicit `--audit-json`
  3. `SDKWORK_RELEASE_SYNC_AUDIT_PATH`
  4. `SDKWORK_RELEASE_SYNC_AUDIT_JSON`
  5. default `docs/release/release-sync-audit-latest.json`
  6. live Git collection

## 3. Contract Effect

- restore now hydrates files that the real CLI entrypoints will actually consume.
- operator recovery path becomes:
  1. download governed artifacts
  2. run `restore-release-governance-latest.mjs`
  3. run `run-release-governance-checks.mjs`
- this matches the repository-owned truth model without requiring special fallback-only execution.

## 4. Honest Boundary

- default latest artifacts do not override explicit governed input.
- default latest artifacts do not fabricate live Git truth.
- live Git still remains the final source only when no governed input or restored latest artifact exists.

## 5. Remaining Closure

- telemetry evidence still needs a stable upstream producer/retention path.
- operator UX can still be improved with a bundled governance artifact or manifest.
