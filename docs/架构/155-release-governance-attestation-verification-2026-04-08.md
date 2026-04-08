# Release Governance Attestation Verification

> Date: 2026-04-08
> Goal: close the operator-side attestation verification gap left after workflow provenance generation was introduced.

## 1. Problem

- `docs/架构/154` made release jobs generate build-provenance attestations.
- The repository still lacked a verification entrypoint for operators or governance tooling.
- That left the architecture asymmetric:
  - workflow could attest
  - repository could not verify

## 2. Verification Subjects

- Governed evidence:
  - `docs/release/release-telemetry-snapshot-latest.json`
  - `docs/release/slo-governance-latest.json`
- Unix smoke evidence:
  - `artifacts/release-governance/unix-installed-runtime-smoke-*.json`
- Packaged release assets:
  - files discovered under `artifacts/release/`

## 3. Verification Command

- Repository entrypoint:
  - `node scripts/release/verify-release-attestations.mjs --format json`
- Underlying verifier:
  - `gh attestation verify <subject> --repo <owner/repo>`
- Default repository slug:
  - `Sdkwork-Cloud/sdkwork-api-router`
- Override:
  - `--repo <owner/repo>`
  - `SDKWORK_RELEASE_ATTESTATION_REPOSITORY`
  - `GITHUB_REPOSITORY`

## 4. Result Semantics

- `subject-path-missing`
  - required local verification subject is absent
- `gh-cli-missing`
  - `gh` is unavailable on the host
- `command-exec-blocked`
  - host policy blocks child execution
- `attestation-verify-failed`
  - `gh attestation verify` ran and rejected the subject

## 5. Governance Integration

- `run-release-governance-checks.mjs` now includes:
  - `release-attestation-verify-test`
- This slice adds only the test lane, not a live lane.
- Rationale:
  - current host still blocks some child processes
  - local session still lacks real hosted attestation records
  - adding a live lane now would create a permanent blocked lane without new release truth

## 6. Fallback Contract

- When Node child execution is blocked, governance runner now falls back to:
  - `scripts/release/release-attestation-verification-contracts.mjs`
- The fallback asserts:
  - script exists
  - exported verification helpers exist
  - repository-owned subject-spec coverage remains intact

## 7. Non-Goals

- Do not claim local end-to-end proof of GitHub-hosted attestation verification.
- Do not convert attestation verification into a live release blocker yet.
- Do not claim the telemetry export producer gap is closed.

## 8. Remaining Closure

- Hosted release execution still needs to produce real attestation records.
- Governed evidence still must be materialized locally before local verification can pass.
- A future slice can decide whether attestation verification should become a live governance lane.
