# 2026-04-08 Release Sync Audit Governed Input Review

## 1. Scope

- `scripts/release/verify-release-sync.mjs`
- `scripts/release/run-release-governance-checks.mjs`
- `scripts/release/release-sync-audit-contracts.mjs`
- `scripts/release/tests/release-sync-audit.test.mjs`
- `scripts/release/tests/release-governance-runner.test.mjs`

## 2. Finding

### P1 release-sync audit still depended only on host-local Git child execution

- After closing governed ingress for `release-window-snapshot`, the remaining Git-blocked live lane was `release-sync-audit`.
- `verify-release-sync.mjs` still had only one execution mode:
  - recompute all repository sync facts locally through Git
- On the current host that kept the lane blocked with `command-exec-blocked`, even when operators could supply an auditable sync summary from an allowed environment.

## 3. Root Cause

- No governed input contract existed for release-sync audit facts.
- `run-release-governance-checks.mjs` fallback replay called `auditReleaseSyncRepositories()` directly, without any environment-backed ingress.
- That made the repository truthful but unnecessarily closed to governed sync evidence, unlike release-window snapshotting and SLO evidence materialization.

## 4. Changes

- Updated `verify-release-sync.mjs`
  - added governed input resolution via:
    - `--audit <path>`
    - `--audit-json <json>`
    - `SDKWORK_RELEASE_SYNC_AUDIT_PATH`
    - `SDKWORK_RELEASE_SYNC_AUDIT_JSON`
  - accepts either:
    - a governed artifact with `generatedAt`, `source`, and `summary`
    - a raw sync-audit summary object with `releasable` and `reports`
  - validates the governed summary and reports before accepting them
  - bypasses live Git execution when governed input is present
- Updated `run-release-governance-checks.mjs`
  - passes `env` into the `release-sync-audit` fallback path so blocked-host replay can consume the same governed input
- Hardened contracts and tests
  - `release-sync-audit-contracts.mjs` now requires governed-input exports and bypass behavior
  - `release-sync-audit.test.mjs` now proves governed JSON input works without live Git
  - `release-governance-runner.test.mjs` now proves the top-level governance runner consumes governed sync-audit input when Node child execution is blocked

## 5. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs`
  - `2 / 2`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `13 / 13`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-window-snapshot.test.mjs`
  - `5 / 5`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - default truth: `7` pass / `3` block / `0` fail
- `node scripts/release/run-release-governance-checks.mjs --format json` with `SDKWORK_RELEASE_SYNC_AUDIT_JSON`
  - `8` pass / `2` block / `0` fail
- `node scripts/release/run-release-governance-checks.mjs --format json` with both:
  - `SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON`
  - `SDKWORK_RELEASE_SYNC_AUDIT_JSON`
  - `9` pass / `1` block / `0` fail

## 6. Current Truth

- This slice closes the governed ingress contract for release-sync audit facts.
- It does not claim the current local host now permits live Git child execution.
- Without governed input, the lane still blocks honestly.
- With governed input, the repository can now consume auditable sync facts instead of degrading blocked hosts into fake PASS states.

## 7. Next Step

1. Close the last remaining blocked live lane by supplying or producing governed telemetry input for `release-slo-governance`.
2. Define hosted production and retention for governed release-sync audit artifacts.
3. Keep default blocked-host truth intact until real governed evidence is supplied.
