# Release Governance Governed Evidence Artifacts

> Date: 2026-04-08
> Goal: persist the governed telemetry snapshot and governed SLO evidence as first-class release-governance artifacts.

## 1. Problem

- The release workflow already materialized:
  - `docs/release/release-telemetry-snapshot-latest.json`
  - `docs/release/slo-governance-latest.json`
- Both files were consumed in-place and then lost with the job workspace.
- That left the release workflow behind the expected audit posture: the release gate could decide, but operators could not retrieve the exact governed evidence bundle that decision used.

## 2. Design

1. Materialize governed telemetry snapshot.
2. Upload the snapshot as a dedicated release-governance artifact.
3. Materialize governed SLO evidence from that snapshot.
4. Upload the SLO evidence as a dedicated release-governance artifact.
5. Run the governance gate.

## 3. Artifact Contract

- Native jobs upload:
  - `release-governance-telemetry-snapshot-${platform}-${arch}`
  - `release-governance-slo-evidence-${platform}-${arch}`
- Web job uploads:
  - `release-governance-telemetry-snapshot-web`
  - `release-governance-slo-evidence-web`
- Paths:
  - `docs/release/release-telemetry-snapshot-latest.json`
  - `docs/release/slo-governance-latest.json`

## 4. Why Separate Artifacts

- Governance evidence must remain separate from `release-assets-*`.
- Customer-facing binaries and internal release proof have different consumers, retention rules, and security posture.
- Distinct artifact names also align with GitHub Actions guidance to persist workflow outputs with explicit names instead of relying on unnamed defaults.

## 5. Benchmark Alignment

- GitHub workflow artifact guidance treats artifacts as the supported retention boundary for build and test outputs after workflow completion.
- GitHub artifact attestation guidance treats released artifacts as provenance targets and ties them to verifiable build instructions.
- This design closes the first gap, evidence retention.
- It intentionally leaves the second gap, cryptographic provenance, for a later slice.

## 6. Non-Goals

- Do not publish governed evidence as release assets.
- Do not claim a real release-time telemetry export producer already exists.
- Do not claim artifact attestation is already implemented.

## 7. Remaining Closure

- `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON` still lacks a repository-owned live producer or control-plane handoff.
- Governance evidence still lacks attestation or signed digest policy.
- `release-window-snapshot` and `release-sync-audit` still depend on a host that allows Git child-process execution.
