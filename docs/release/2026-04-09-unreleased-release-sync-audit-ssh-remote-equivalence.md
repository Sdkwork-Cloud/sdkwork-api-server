# Unreleased - Release Sync Audit SSH Remote Equivalence

- Date: 2026-04-09
- Type: patch
- Scope: Step 11 / Step 13 release-governance accuracy
- Highlights:
  - normalized GitHub remote comparison in `verify-release-sync.mjs` so SSH and HTTPS origins for the same repository no longer produce false `remote-url-mismatch` failures
  - added a red-first regression for `git@github.com:Sdkwork-Cloud/sdkwork-api-router.git` equivalence and re-verified the adjacent release-governance suites
- Verification:
  - `release-sync-audit.test.mjs`: `3 / 3`
  - `materialize-release-sync-audit.test.mjs`: `3 / 3`
  - `release-governance-runner.test.mjs`: `16 / 16`
  - `run-release-governance-checks.mjs --format json`: `7` pass / `3` block / `0` fail
- Known gaps:
  - local governance still blocks on Node child-Git `EPERM` and missing telemetry input
  - this workspace is still ahead/dirty, so release sync is not yet releasable even after URL-equivalence correction
