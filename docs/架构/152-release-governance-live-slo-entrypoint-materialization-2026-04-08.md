# Release Governance Live SLO Entrypoint Materialization

> Date: 2026-04-08
> Goal: make `run-release-governance-checks.mjs` replay the same live SLO materialization chain that the release workflow already declares.

## 1. Problem

- The release workflow already staged:
  1. telemetry export input
  2. governed telemetry snapshot
  3. governed SLO evidence
  4. live SLO evaluation
- The repository single entrypoint skipped steps `1-3` and jumped directly to step `4`.
- That mismatch made local/live governance replay weaker than the architecture contract.

## 2. Design

When live SLO evidence is missing, `run-release-governance-checks.mjs` must:

1. reuse `docs/release/release-telemetry-snapshot-latest.json` if it already exists
2. otherwise materialize it from governed telemetry input
3. materialize `docs/release/slo-governance-latest.json`
4. evaluate `release-slo-governance`

## 3. Classification Rule

- No telemetry input available:
  - `blocked=true`
  - `reason=telemetry-input-missing`
- Telemetry/SLO materialization is malformed:
  - fail the lane
- Materialization succeeds and objectives pass:
  - `release-slo-governance` passes

## 4. Why This Matters

- `run-release-governance-checks.mjs` is the documented single release-truth entrypoint.
- Operators should not need to manually replay hidden prerequisite scripts before that entrypoint tells the truth.
- This keeps the release workflow and local governance replay aligned without committing fake latest artifacts.

## 5. Non-Goals

- Do not commit synthetic `release-telemetry-snapshot-latest.json`.
- Do not commit synthetic `slo-governance-latest.json`.
- Do not claim the repository now has a real production telemetry export producer.

## 6. Remaining Closure

- A real producer/handoff for `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON` is still missing.
- Telemetry snapshot and SLO evidence are still not uploaded as dedicated release-governance artifacts.
- `release-window-snapshot` and `release-sync-audit` still block on host Git child-process policy.
