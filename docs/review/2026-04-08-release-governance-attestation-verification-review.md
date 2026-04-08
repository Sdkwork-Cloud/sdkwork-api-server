# 2026-04-08 Release Governance Attestation Verification Review

## 1. Scope

- `scripts/release/verify-release-attestations.mjs`
- `scripts/release/release-attestation-verification-contracts.mjs`
- `scripts/release/run-release-governance-checks.mjs`
- `scripts/release/tests/release-attestation-verify.test.mjs`
- `scripts/release/tests/release-governance-runner.test.mjs`

## 2. Finding

### P1 release workflow could generate artifact attestations, but the repository still lacked a verification entrypoint

- `docs/架构/154` had already closed provenance generation in the workflow.
- The repository still had no owned command to verify governed evidence or packaged assets with `gh attestation verify`.
- Result: workflow truth and operator truth were still split.

## 3. Root Cause

- Attestation generation existed only in `.github/workflows/release.yml`.
- There was no repository-owned subject discovery policy for:
  - governed telemetry snapshot
  - governed SLO evidence
  - Unix smoke evidence
  - packaged release assets
- There was no contract/fallback lane in `run-release-governance-checks.mjs` for this capability.

## 4. Design

1. Add `scripts/release/verify-release-attestations.mjs`.
2. Keep subject discovery repository-owned and explicit.
3. Distinguish:
   - `subject-path-missing`
   - `gh-cli-missing`
   - `command-exec-blocked`
   - real `attestation-verify-failed`
4. Add only a test lane to governance runner:
   - `release-attestation-verify-test`
5. Do not add a new live verification lane yet.

## 5. Changes

- Added `scripts/release/verify-release-attestations.mjs`
  - discovers governed evidence, Unix smoke evidence, and packaged release assets
  - builds `gh attestation verify <subject> --repo <slug>` command plans
  - returns structured blocked/fail/pass summaries
- Added `scripts/release/release-attestation-verification-contracts.mjs`
  - locks script exports and subject-spec coverage into an in-process fallback contract
- Updated `scripts/release/run-release-governance-checks.mjs`
  - inserted `release-attestation-verify-test`
  - added fallback support for child-exec-restricted hosts
- Added `scripts/release/tests/release-attestation-verify.test.mjs`
  - subject discovery and plan coverage
  - missing-subject blocked coverage
  - blocked `gh` execution coverage
  - real verify-failure coverage
- Updated `scripts/release/tests/release-governance-runner.test.mjs`
  - fixed lane order expectations
  - added attestation-verification fallback proof

## 6. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-attestation-verify.test.mjs`
  - `4 / 4`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `11 / 11`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
  - `13 / 13`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - `passingIds`: `7`
  - `blockedIds`: `3`
  - `failingIds`: `0`
- `node scripts/release/verify-release-attestations.mjs --format json`
  - blocked truth remained honest:
    - governed snapshot missing
    - governed SLO evidence missing
    - Unix smoke evidence missing
    - local `gh` execution blocked on this host for discovered packaged assets

## 7. Current Truth

- The repository now owns both sides of the attestation story:
  - generation in workflow
  - verification entrypoint in repo
- Governance runner now proves this verification capability even when Node child execution is blocked.
- This local session still did not verify real hosted GitHub attestation records end to end.

## 8. Next Step

1. Capture the first hosted release run that produces real attestation records.
2. Materialize governed snapshot and SLO evidence before running local verification.
3. Continue with the remaining live telemetry export producer/control-plane handoff gap.
