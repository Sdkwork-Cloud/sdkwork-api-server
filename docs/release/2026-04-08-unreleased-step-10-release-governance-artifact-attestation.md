# Unreleased - Step 10 Release Governance Artifact Attestation

- Date: 2026-04-08
- Type: patch
- Summary:
  - updated `.github/workflows/release.yml` so release jobs now generate build-provenance attestations for governed telemetry/SLO evidence, Unix smoke evidence, and packaged release assets
  - added `id-token`, `attestations`, and `artifact-metadata` workflow permissions required by the official attestation action
  - gated attestation by repository support rules: public repos attest automatically, private/internal repos require `SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED == 'true'`
  - hardened release-workflow contracts and tests so removing permissions or attestation steps now breaks release verification immediately
- Verification:
  - `release-workflow.test.mjs`: `13 / 13`
  - `release-governance-runner.test.mjs`: `10 / 10`
  - default governance summary: `6` pass / `3` block / `0` fail
- Remaining truth:
  - this local session did not execute a hosted GitHub attestation step directly
  - the repository still does not own a real release-time telemetry export producer
  - Git-policy-blocked live lanes remain follow-up work
