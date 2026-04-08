# Unreleased - Step 10 Release Windows Installed Runtime Smoke Parity

- Date: 2026-04-08
- Type: patch
- Summary:
  - added `scripts/release/run-windows-installed-runtime-smoke.mjs` so Windows native release lanes can install real built outputs into a runtime home, execute installed `start.ps1`, probe unified health endpoints, execute installed `stop.ps1`, and persist JSON smoke evidence
  - rewired `.github/workflows/release.yml` so Windows native release lanes now run the smoke gate, upload `release-governance-windows-installed-runtime-smoke-*`, and generate a dedicated smoke evidence attestation before packaging release assets
  - extended `scripts/release/verify-release-attestations.mjs` and `scripts/release/release-attestation-verification-contracts.mjs` so Windows smoke evidence is part of repository-owned attestation verification truth
- Verification:
  - `run-windows-installed-runtime-smoke.test.mjs`: `2 / 2`
  - `release-attestation-verify.test.mjs`: `4 / 4`
  - `release-workflow.test.mjs`: `13 / 13`
  - `release-governance-runner.test.mjs`: `11 / 11`
  - governance summary: `7` pass / `3` block / `0` fail
- Remaining truth:
  - local session verified the script/workflow/governance contracts, not a hosted end-to-end Windows smoke run
  - `release-slo-governance`, `release-window-snapshot`, and `release-sync-audit` remain blocked for the same live-evidence / host-policy reasons as before
  - the next real closure point is hosted Windows smoke evidence plus successful attestation verification against that evidence
