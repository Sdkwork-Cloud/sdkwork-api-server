# Unreleased - Step 10 Release Governance Governed Evidence Artifacts

- Date: 2026-04-08
- Type: patch
- Summary:
  - updated `.github/workflows/release.yml` so both release jobs now upload governed telemetry snapshot and governed SLO evidence as dedicated `release-governance-*` artifacts
  - kept those uploads separate from `release-assets-*`, so internal release proof is no longer mixed with customer-facing deliverables
  - hardened `scripts/release/release-workflow-contracts.mjs` and `scripts/release/tests/release-workflow.test.mjs` so removing either governed evidence upload now breaks release verification immediately
  - added rejection coverage for missing telemetry snapshot uploads and missing SLO evidence uploads in addition to the positive workflow proof
- Verification:
  - `release-workflow.test.mjs`: `11 / 11`
  - `release-governance-runner.test.mjs`: `10 / 10`
  - default governance summary: `6` pass / `3` block / `0` fail
- Remaining truth:
  - the repository still does not own a real release-time telemetry export producer
  - the workflow still does not generate artifact attestations for release assets or governance evidence
  - Git-policy-blocked live lanes remain follow-up work
