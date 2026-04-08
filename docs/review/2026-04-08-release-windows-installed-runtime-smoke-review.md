# 2026-04-08 Release Windows Installed Runtime Smoke Review

## 1. Scope

- `.github/workflows/release.yml`
- `scripts/release/run-windows-installed-runtime-smoke.mjs`
- `scripts/release/verify-release-attestations.mjs`
- `scripts/release/release-attestation-verification-contracts.mjs`
- `scripts/release/release-workflow-contracts.mjs`
- `scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs`
- `scripts/release/tests/release-attestation-verify.test.mjs`
- `scripts/release/tests/release-workflow.test.mjs`

## 2. Finding

### P0 Windows release lanes still lacked installed-runtime parity with Unix release truth

- `docs/架构/143-*` and `docs/架构/144-*` already required cross-platform installed-runtime verification.
- Unix lanes already had a real release gate plus persisted smoke evidence.
- Windows lanes still stopped at build output and desktop packaging, so release truth did not yet prove `install -> start -> health -> stop` on the Windows install layout.

## 3. Root Cause

- Earlier runtime-tooling and Unix smoke slices closed repository contracts first.
- No repository-owned Windows smoke entrypoint existed for the release workflow.
- Attestation verification also had no governed subject for Windows smoke evidence, so even uploaded evidence would not have been part of the operator verification surface.

## 4. Changes

- Added `scripts/release/run-windows-installed-runtime-smoke.mjs`
  - parses explicit `--platform`, `--arch`, `--target`, `--runtime-home`, `--evidence-path`
  - rejects non-Windows lanes honestly
  - reuses `createInstallPlan` and `applyInstallPlan`
  - rewrites `config/router.env` to loopback ports
  - runs installed `start.ps1`
  - probes:
    - `/api/v1/health`
    - `/api/admin/health`
    - `/api/portal/health`
  - runs installed `stop.ps1`
  - emits structured JSON evidence for both success and failure
- Rewired `.github/workflows/release.yml`
  - added `Run installed native runtime smoke on Windows`
  - added `Upload Windows installed runtime smoke evidence`
  - added `Generate Windows smoke evidence attestation`
  - kept the Windows slice ordered after native desktop builds and before native asset packaging
- Extended attestation verification
  - `verify-release-attestations.mjs` now treats `windows-installed-runtime-smoke-*.json` as a governed verification subject
  - `release-attestation-verification-contracts.mjs` now requires the Windows smoke subject alongside telemetry, SLO, Unix smoke, and release assets

## 5. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs`
  - `2 / 2`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-attestation-verify.test.mjs`
  - `4 / 4`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
  - `13 / 13`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `11 / 11`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - `passingIds`: `7`
  - `blockedIds`: `3`
  - `failingIds`: `0`

## 6. Current Truth

- Windows release lanes now have repository-owned installed-runtime smoke wiring, persisted governance evidence, and attestation coverage.
- This local session verified workflow contracts, script contracts, and governance integration.
- This local session did not claim a full hosted Windows `start.ps1` / `stop.ps1` smoke execution end to end because the current host can block `Node -> powershell.exe` child execution.
- Current blocked governance lanes remain unchanged:
  - `release-slo-governance`: missing live telemetry input
  - `release-window-snapshot`: Git child exec blocked by host `EPERM`
  - `release-sync-audit`: Git child exec blocked by host `EPERM`

## 7. Next Step

1. Collect the first hosted Windows smoke evidence artifact from a PowerShell-capable release lane.
2. Verify the hosted Windows smoke attestation with `gh attestation verify` when governed evidence is present.
3. Continue replacing host-blocked fallback lanes with real hosted release evidence instead of local fallback-only proof.
