# Release Telemetry Export Producer Governance

> Date: 2026-04-08
> Goal: close the producer boundary upstream of the governed release telemetry snapshot without shrinking the existing SLO baseline.

## 1. Problem

- `release-telemetry-snapshot-latest.json` already existed as a governed artifact.
- The workflow still treated `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON` as upstream ingress.
- That mixed raw telemetry handoff with governed snapshot truth and left no repository-owned export contract.

## 2. Export Contract

- The governed upstream input is now a release telemetry export bundle.
- Supported ingress:
  - `--export`
  - `--export-json`
  - `SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH`
  - `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`
- Bundle shape:
  - `generatedAt`
  - `source.kind`
  - `prometheus.gateway`
  - `prometheus.admin`
  - `prometheus.portal`
  - `supplemental.targets`

## 3. Derivation Rules

- Direct derivation from raw Prometheus text is currently limited to:
  - `gateway-availability`
  - `admin-api-availability`
  - `portal-api-availability`
- Source metric:
  - `sdkwork_http_requests_total`
- Availability rule:
  - numerator: all non-`5xx` requests
  - denominator: all requests
- Burn-rate rule:
  - compute from the governed SLO objective already defined in `scripts/release/slo-governance.mjs`
- Remaining targets stay in `supplemental.targets`.

## 4. Why Mixed Derivation Is Required

- The observability crate currently exposes request totals and duration sum/count pairs.
- It does not expose histogram/bucket series that would let the repo derive p95 latency honestly from raw Prometheus text.
- Therefore the repo must not pretend that targets such as `routing-simulation-p95-latency` are directly derivable today.

## 5. Workflow Contract

1. `Materialize external release dependencies`
2. `Materialize release telemetry snapshot`
3. `Materialize SLO governance evidence`
4. `Run release governance gate`

The snapshot step now expects `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`, while the SLO step still consumes the governed snapshot artifact path.

## 6. Non-Goals

- Do not reduce the `14`-target SLO baseline to fit current raw metrics.
- Do not commit synthetic latest snapshot/evidence artifacts as repository truth.
- Do not claim full live telemetry closure before a real export producer exists.

## 7. Remaining Closure

- A real release-time export producer is still missing.
- Freshness and provenance policy are still operational controls, not repository-enforced release truth yet.
- Direct derivation can expand only after raw metrics are expanded.
