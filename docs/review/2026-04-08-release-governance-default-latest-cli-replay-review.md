# 2026-04-08 Release Governance Default Latest CLI Replay Review

## 1. Scope

- `scripts/release/compute-release-window-snapshot.mjs`
- `scripts/release/verify-release-sync.mjs`
- `scripts/release/tests/release-window-snapshot.test.mjs`
- `scripts/release/tests/release-sync-audit.test.mjs`

## 2. Finding

### P1 restore contract only closed fallback replay, not the real CLI path

- `restore-release-governance-latest.mjs` already wrote:
  - `docs/release/release-window-snapshot-latest.json`
  - `docs/release/release-sync-audit-latest.json`
- `run-release-governance-checks.mjs --format json` on this host still blocked before this fix because:
  - Node child execution was allowed
  - child scripts then tried live Git
  - live Git hit `EPERM`
- Result:
  - restore proof existed inside fallback-only tests
  - the operator-facing CLI path still did not honor restored latest artifacts for these two lanes

## 3. Root Cause

- `compute-release-window-snapshot.mjs` input order was:
  1. explicit `--snapshot` / env input
  2. live Git
- `verify-release-sync.mjs` input order was:
  1. explicit `--audit` / env input
  2. live Git
- Neither script auto-discovered the repository-owned default latest files written by restore.
- Therefore the restore contract from architecture docs `161` and `162` was only partially implemented.

## 4. Changes

- updated `compute-release-window-snapshot.mjs`
  - explicit CLI/env input still wins
  - default `docs/release/release-window-snapshot-latest.json` is now second
  - live Git is now last
- updated `verify-release-sync.mjs`
  - explicit CLI/env input still wins
  - default `docs/release/release-sync-audit-latest.json` is now second
  - live Git is now last
- added regression tests:
  - default release-window latest artifact is consumed before live Git
  - default release-sync latest artifact is consumed before live Git

## 5. Verification

- red-first:
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-window-snapshot.test.mjs`
    - failed on `gitSpawned === true`
  - `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs`
    - failed on `gitSpawned === true`
- green:
  - `release-window-snapshot.test.mjs`
    - `6 / 6`
  - `release-sync-audit.test.mjs`
    - `3 / 3`
- operator CLI proof after restoring governed artifacts:
  - `node scripts/release/restore-release-governance-latest.mjs --artifact-dir tmp/release-governance-cli-sim`
    - restored all `5` required latest files
  - `node scripts/release/run-release-governance-checks.mjs --format json`
    - `ok=true`
    - `blocked=false`
    - `failingIds=[]`

## 6. Current Truth

- blocked hosts now have a real end-to-end path:
  1. download governed artifacts
  2. restore latest files
  3. run governance CLI
- default local truth is still honestly blocked when no latest artifacts or telemetry evidence exist.
- no synthetic release truth was committed to `docs/release/`.

## 7. Next Step

1. Add an optional bundled governance artifact or manifest so operators do not have to download multiple artifacts manually.
2. Keep upstream telemetry evidence supply as a separate closure item; this fix only closes replay consumption, not evidence production.
