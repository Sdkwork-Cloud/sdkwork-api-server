# Unreleased - Step 10 Release Governance Attestation Verification

- Date: 2026-04-08
- Type: patch
- Summary:
  - added `scripts/release/verify-release-attestations.mjs` so the repository now owns a verification entrypoint for governed evidence, Unix smoke evidence, and packaged release assets
  - added `scripts/release/release-attestation-verification-contracts.mjs` and wired `release-attestation-verify-test` into `run-release-governance-checks.mjs`
  - kept the design honest by adding only a test lane, not a new live blocked lane
- Verification:
  - `release-attestation-verify.test.mjs`: `4 / 4`
  - `release-governance-runner.test.mjs`: `11 / 11`
  - `release-workflow.test.mjs`: `13 / 13`
  - default governance summary: `7` pass / `3` block / `0` fail
- Remaining truth:
  - local verification still blocks until governed evidence is materialized and host `gh` execution is available
  - this local session did not verify a real hosted GitHub attestation record end to end
  - `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON` still lacks a repository-owned live producer/control-plane handoff
