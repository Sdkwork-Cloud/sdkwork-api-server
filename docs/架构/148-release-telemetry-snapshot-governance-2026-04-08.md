# Release Telemetry Snapshot Governance

> Date: 2026-04-08
> Goal: close the missing producer contract between external telemetry and the release SLO gate without pretending live telemetry is already solved.

## 1. Problem

- `release-slo-governance` existed.
- `materialize-slo-governance-evidence.mjs` existed.
- The release workflow still fed that step with raw `SDKWORK_SLO_GOVERNANCE_EVIDENCE_JSON`.

That meant the gate had an evaluator but no repository-owned telemetry ingress contract.

## 2. Design

- Add a governed release telemetry snapshot artifact:
  - path: `docs/release/release-telemetry-snapshot-latest.json`
  - producer: `scripts/release/materialize-release-telemetry-snapshot.mjs`
  - inputs:
    - file: `--snapshot` or `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH`
    - JSON: `--snapshot-json` or `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON`
- Keep SLO evidence as a second governed artifact:
  - path: `docs/release/slo-governance-latest.json`
  - producer: `scripts/release/materialize-slo-governance-evidence.mjs`
  - workflow input: `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH`

## 3. Contract

- Snapshot artifact requires:
  - `generatedAt`
  - `source.kind`
  - `targets`
- `targets` must cover the same `14` governed SLO targets already defined in `scripts/release/slo-governance.mjs`.
- Each target must carry:
  - `ratio` or `value`, depending on indicator type
  - `burnRates.1h`
  - `burnRates.6h`

This keeps the snapshot rich enough for audit while staying aligned with the existing SLO baseline.

## 4. Workflow Order

1. `Materialize external release dependencies`
2. `Materialize release telemetry snapshot`
3. `Materialize SLO governance evidence`
4. `Run release governance gate`

The snapshot step is now the fixed input boundary. The SLO step is no longer the raw ingress point.

## 5. Non-Goals

- Do not commit a fake `release-telemetry-snapshot-latest.json`.
- Do not claim real telemetry export is solved.
- Do not bypass the SLO gate when snapshot/evidence is missing.

## 6. Remaining Closure

- The workflow still needs a real producer for `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON`.
- Freshness, ownership, and audit trail are still operational decisions, not repository truths yet.
- The release gate must remain blocking until that producer is wired.
