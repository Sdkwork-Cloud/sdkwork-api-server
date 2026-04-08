# Release Governance Artifact Attestation

> Date: 2026-04-08
> Goal: raise retained release artifacts from audit evidence to cryptographically verifiable provenance targets.

## 1. Problem

- The workflow already retained:
  - governed telemetry snapshot
  - governed SLO evidence
  - Unix installed-runtime smoke evidence
  - packaged release assets
- Those artifacts were retrievable, but not attested.
- That left the workflow behind the GitHub supply-chain baseline for released artifacts.

## 2. Design

1. Keep artifact retention.
2. Add build-provenance attestation for retained governance evidence.
3. Add build-provenance attestation for packaged release assets.
4. Keep attestation outside `release-assets-*`; attestations are provenance, not shipped payload.

## 3. Attestation Rule

- Public repositories:
  - run attestation automatically
- Private or internal repositories:
  - run attestation only when `SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED == 'true'`
- Rationale:
  - GitHub documents that public repositories on all current plans can use artifact attestations
  - private/internal repositories require GitHub Enterprise Cloud
  - the workflow cannot safely infer the billing plan

## 4. Subjects

- Native jobs:
  - `docs/release/release-telemetry-snapshot-latest.json`
  - `docs/release/slo-governance-latest.json`
  - `artifacts/release-governance/unix-installed-runtime-smoke-${platform}-${arch}.json` on Unix only
  - `artifacts/release/**/*`
- Web job:
  - `docs/release/release-telemetry-snapshot-latest.json`
  - `docs/release/slo-governance-latest.json`
  - `artifacts/release/**/*`

## 5. Workflow Order

- Governed evidence must remain:
  1. materialized
  2. uploaded
  3. attested
  4. evaluated by the governance gate
- Release assets must remain:
  1. packaged
  2. uploaded
  3. attested

## 6. Security Contract

- Required permissions:
  - `contents: write`
  - `id-token: write`
  - `attestations: write`
  - `artifact-metadata: write`
- Action:
  - `actions/attest-build-provenance@v3`

## 7. Why `attest-build-provenance`

- GitHub Docs and the current marketplace entry still document `actions/attest-build-provenance@v3` as the direct path for build provenance.
- The repository uses that documented path to minimize workflow risk in this slice.
- Future migration to `actions/attest` remains possible if the repository wants the newer generic attestation surface.

## 8. Non-Goals

- Do not claim this local session executed a hosted attestation successfully.
- Do not claim artifact verification by consumers is already operationalized.
- Do not claim the telemetry export producer gap is closed.

## 9. Remaining Closure

- First hosted release run still needs to produce real attestation records.
- Operator verification with `gh attestation verify` still needs a documented release-playbook walkthrough.
- `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON` still lacks a repository-owned live producer or control-plane handoff.
