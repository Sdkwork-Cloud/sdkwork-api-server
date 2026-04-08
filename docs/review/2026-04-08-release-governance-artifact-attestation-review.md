# 2026-04-08 Release Governance Artifact Attestation Review

## 1. Scope

- `.github/workflows/release.yml`
- `scripts/release/release-workflow-contracts.mjs`
- `scripts/release/tests/release-workflow.test.mjs`

## 2. Finding

### P1 release governance retained artifacts but still lacked cryptographic provenance

- The release workflow already persisted:
  - governed telemetry snapshot
  - governed SLO evidence
  - Unix installed-runtime smoke evidence
  - packaged release assets
- None of those retained artifacts had workflow-linked build provenance.
- Result: audit retention existed, but supply-chain verifiability still lagged the GitHub artifact-attestation baseline.

## 3. Root Cause

- The workflow had no attestation permissions.
- The workflow had no build-provenance steps for governance evidence or packaged release assets.
- The repository also needed a plan-aware guard because GitHub documents different artifact-attestation availability for public and private/internal repositories.

## 4. Design

1. Add workflow permissions:
   - `contents: write`
   - `id-token: write`
   - `attestations: write`
   - `artifact-metadata: write`
2. Auto-enable attestation for public repositories.
3. For private/internal repositories, only run attestation when `SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED == 'true'`.
4. Attest:
   - governed telemetry snapshot + governed SLO evidence
   - Unix smoke evidence on Unix native lanes
   - packaged native release assets
   - packaged web release assets

## 5. Why The Guard Exists

- GitHub documents that artifact attestations are available on all current plans for public repositories.
- For private or internal repositories, GitHub requires GitHub Enterprise Cloud.
- The repository cannot infer billing plan safely inside the workflow.
- Therefore the safest repository-owned rule is:
  - public repo: attest automatically
  - private/internal repo: operator opt-in via explicit variable

## 6. Changes

- Updated `.github/workflows/release.yml`
  - added attestation permissions
  - added `Generate governance evidence attestation` in native and web jobs
  - added `Generate Unix smoke evidence attestation` on Unix native lanes
  - added `Generate native release assets attestation`
  - added `Generate web release assets attestation`
  - all attestation steps use `actions/attest-build-provenance@v3`
  - all attestation steps are gated by:
    - public repository, or
    - `SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED == 'true'`
- Updated `scripts/release/release-workflow-contracts.mjs`
  - workflow permissions are now part of executable release truth
  - attestation steps and their order are now part of executable release truth
- Updated `scripts/release/tests/release-workflow.test.mjs`
  - added positive proof for permissions and attestation steps
  - added rejection proof for missing attestation permissions
  - added rejection proof for missing attestation steps
## 7. Industry Alignment

- GitHub positions artifact attestations as the provenance layer for build outputs and describes them as cryptographically signed claims that link artifacts to workflow identity.
  - Sources:
    - https://docs.github.com/en/actions/concepts/security/artifact-attestations
    - https://docs.github.com/en/enterprise-cloud@latest/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations
- GitHub documents plan availability explicitly:
  - public repos on all current plans
  - private/internal repos require GitHub Enterprise Cloud
  - same source as above
- The official `actions/attest-build-provenance` README also requires `id-token: write` and `attestations: write`, and notes `artifact-metadata: write` enables storage records.
  - Source: https://github.com/actions/attest-build-provenance

## 8. Verification

- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
  - `13 / 13`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
  - `10 / 10`
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - `passingIds`: `6`
  - `blockedIds`: `3`
  - `failingIds`: `0`
  - unchanged live blockers:
    - `release-slo-governance`: `telemetry-input-missing`
    - `release-window-snapshot`: `command-exec-blocked`
    - `release-sync-audit`: `command-exec-blocked`

## 9. Current Truth

- The repository now encodes build-provenance generation for release assets and governed evidence into the release workflow contract.
- This local session did not execute GitHub-hosted attestation steps directly, so hosted-runner proof still belongs to the next release execution.
- The repository still does not own a real release-time telemetry export producer.

## 10. Next Step

1. Run the release workflow on a supported GitHub-hosted environment and capture the first real attestation evidence.
2. Add operator-facing verification guidance around `gh attestation verify` for published release assets.
3. Continue with the missing real telemetry export producer or control-plane handoff.
