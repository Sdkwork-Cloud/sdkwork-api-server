# 2026-04-08 Release Window / Sync Governed Latest Artifacts Review

## 1. Scope

- `scripts/release/materialize-release-window-snapshot.mjs`
- `scripts/release/materialize-release-sync-audit.mjs`
- `scripts/release/verify-release-sync.mjs`
- `scripts/release/run-release-governance-checks.mjs`
- `scripts/release/verify-release-attestations.mjs`
- `scripts/release/release-attestation-verification-contracts.mjs`
- `scripts/release/release-workflow-contracts.mjs`
- `.github/workflows/release.yml`
- `scripts/release/tests/*`

## 2. Finding

### P1 `release-window-snapshot` and `release-sync-audit` still had governed ingress but no repository-owned latest artifact production chain

- Earlier slices added governed input contracts.
- The remaining gap was operational:
  - no standard producer writing `docs/release/release-window-snapshot-latest.json`
  - no standard producer writing `docs/release/release-sync-audit-latest.json`
  - no workflow upload or attestation coverage for those artifacts
  - blocked-host governance replay could not consume a repository-owned default latest artifact

## 3. Root Cause

- The repository could validate governed facts, but it still treated them as ad hoc inputs instead of first-class release evidence.
- That left the release evidence chain asymmetric:
  - live Git facts existed
  - governed ingress existed
  - durable latest artifact production and attestation did not

## 4. Changes

- Added `materialize-release-window-snapshot.mjs`
  - accepts governed `--snapshot` / `--snapshot-json`
  - otherwise derives fresh facts from live Git
  - writes `docs/release/release-window-snapshot-latest.json`
- Added `materialize-release-sync-audit.mjs`
  - accepts governed `--audit` / `--audit-json`
  - otherwise derives fresh multi-repository sync facts from live Git
  - writes `docs/release/release-sync-audit-latest.json`
- Extended `verify-release-sync.mjs`
  - accepts injectable `spawnSyncImpl` so producer tests and fallback replay can verify live Git behavior deterministically
- Updated `run-release-governance-checks.mjs`
  - fallback replay now prefers explicit governed env input
  - if absent, it replays repository-owned default latest artifacts for release-window and release-sync before declaring blocked
- Updated `.github/workflows/release.yml`
  - native and web release jobs now materialize and upload both latest artifacts
  - governance attestation now includes both artifacts
  - governance gate now consumes the materialized artifact paths explicitly
- Updated attestation verification
  - `release-window-snapshot`
  - `release-sync-audit`
  are now required governed subjects

## 5. Verification

- `materialize-release-window-snapshot.test.mjs`
  - `3 / 3`
- `materialize-release-sync-audit.test.mjs`
  - `3 / 3`
- `release-window-snapshot.test.mjs`
  - `5 / 5`
- `release-sync-audit.test.mjs`
  - `2 / 2`
- `release-governance-runner.test.mjs`
  - `16 / 16`
- `release-attestation-verify.test.mjs`
  - `4 / 4`
- `release-workflow.test.mjs`
  - `13 / 13`

## 6. Current Truth

- The repository now owns both latest-artifact producers and their workflow / attestation contracts.
- Blocked-host replay can consume:
  - explicit governed env input
  - repository-owned default latest artifacts when they exist
- Default local truth is still honest:
  - no synthetic latest artifacts are committed
  - `release-slo-governance` still blocks without telemetry input
  - local default `release-window-snapshot` and `release-sync-audit` still block until latest artifacts are supplied or regenerated on an allowed host

## 7. Next Step

1. Define operator retention and retrieval flow for latest release-window and release-sync artifacts across blocked hosts.
2. Close the remaining `release-slo-governance` upstream telemetry producer / retention gap.
3. Keep local default truth blocked until governed evidence actually exists.
