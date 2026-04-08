# Release Telemetry Export Control-Plane Handoff

> Date: 2026-04-08
> Goal: promote release telemetry export to a first-class governed artifact that the workflow, attestation verifier, and blocked-host replay can all consume consistently.

## 1. Problem

- The repo already owned:
  - governed telemetry snapshot
  - governed SLO evidence
  - workflow evidence upload
  - evidence attestation verification
- It still did not own the upstream export artifact path.
- Result:
  - workflow input stayed env-shaped
  - attestation skipped the export boundary
  - blocked-host replay could not reuse a default export artifact

## 2. Artifact Contract

- Standard path:
  - `docs/release/release-telemetry-export-latest.json`
- Standard envelope:
  - `version`
  - `generatedAt`
  - `source`
  - `prometheus.gateway`
  - `prometheus.admin`
  - `prometheus.portal`
  - `supplemental.targets`

## 3. Producer Contract

- Entry:
  - `scripts/release/materialize-release-telemetry-export.mjs`
- Direct governed input:
  - `--export`
  - `--export-json`
  - `SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH`
  - `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`
- Control-plane handoff assembly:
  - gateway/admin/portal Prometheus text or path
  - supplemental target JSON or path
  - `generatedAt`
  - `source.kind`
  - `source.provenance`
  - `source.freshnessMinutes`

## 4. Workflow Contract

1. `Materialize external release dependencies`
2. `Materialize release telemetry export`
3. `Upload release telemetry export governance artifact`
4. `Materialize release telemetry snapshot`
5. `Upload release telemetry snapshot governance artifact`
6. `Materialize SLO governance evidence`
7. `Upload SLO governance evidence artifact`
8. `Generate governance evidence attestation`
9. `Run release governance gate`

- Snapshot step input:
  - `SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH=docs/release/release-telemetry-export-latest.json`

## 5. Replay Contract

- `materialize-release-telemetry-snapshot.mjs` now auto-discovers:
  - `docs/release/release-telemetry-export-latest.json`
- `run-release-governance-checks.mjs` therefore replays:
  - default export artifact -> snapshot -> SLO evidence -> SLO evaluation
- This improves blocked-host replay without claiming live telemetry exists by default.

## 6. Attestation Contract

- Required governed evidence subjects now include:
  - `release-telemetry-export`
  - `release-telemetry-snapshot`
  - `release-slo-governance`

## 7. Non-Goals

- Do not claim this host now has live telemetry export input by default.
- Do not fake green when no export artifact or handoff input exists.
- Do not reduce the SLO baseline to fit narrower telemetry.

## 8. Remaining Closure

- Hosted release environments still need a stable upstream producer for the control-plane inputs that feed this export step.
- `release-window-snapshot` and `release-sync-audit` still depend on governed input or an allowed Git-exec host.
