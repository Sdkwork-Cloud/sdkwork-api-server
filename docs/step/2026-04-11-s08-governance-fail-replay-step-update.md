# 2026-04-11 S08 Governance Fail Replay Step Update

## Scope

- Step: `S08 integrated acceptance, release gate, and continuous iteration`
- Loop target: close the remaining governed telemetry-input gap truthfully, materialize the default release telemetry latest artifacts, and convert `release-slo-governance` from `blocked` into an evidence-backed quantitative result
- Boundaries:
  - `scripts/release/materialize-release-telemetry-snapshot.mjs`
  - `scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
  - `artifacts/runtime/dev-s08-telemetry-check/release-input/local-governance-fail-replay-live.json`
  - `artifacts/runtime/dev-s08-telemetry-check/release-input/supplemental-targets-local-governance-fail-replay.json`
  - `artifacts/runtime/dev-s08-telemetry-check/release-input/release-telemetry-export-local-governance-fail-replay.json`
  - `artifacts/runtime/dev-s08-telemetry-check/release-input/release-telemetry-snapshot-local-governance-fail-replay.json`
  - `artifacts/runtime/dev-s08-telemetry-check/release-input/slo-governance-local-governance-fail-replay.json`
  - `docs/release/release-telemetry-export-latest.json`
  - `docs/release/release-telemetry-snapshot-latest.json`
  - `docs/release/slo-governance-latest.json`

## Root Cause Investigation

- the previous `S08` blocker was no longer a missing runtime path:
  - a live local stateful-provider probe can truthfully trigger `gateway_execution_failover`
  - that same probe now emits all three evidence classes required for fallback closure:
    - `RoutingDecisionLog`
    - `ProviderHealthSnapshot`
    - provider-level gateway metrics including `sdkwork_gateway_failovers_total`
- the next blocker turned out to be inside the governance materializer itself:
  - `materialize-release-telemetry-snapshot.mjs` parsed Prometheus sample lines with a single regex that stopped at the first `}` character
  - current admin `/metrics` exposes route labels such as `/admin/runtime-config/rollouts/{rollout_id}`
  - that made truthful live telemetry export replay fail before the quantitative gate could even evaluate
- TDD replay now proves that exact root cause:
  - added a failing test where `sdkwork_http_requests_total` carries `route="/admin/runtime-config/rollouts/{rollout_id}"`
  - observed the expected red failure
  - replaced the line splitter with a quote-aware parser that keeps inner `{}` inside label values from terminating the outer sample parse
  - reran the snapshot suite to green

## Changes

- fixed the governed release telemetry snapshot parser so live Prometheus handoff with route-template label values can be materialized truthfully
- executed a new live local governance replay that extends the earlier derived-target probe:
  - `gateway-fallback-success-rate`: now truthfully closed at `1/1`
  - `gateway-provider-timeout-budget`: now truthfully derived at `0/1` for the controlled failover probe window
  - `account-hold-creation-success-rate`: materialized as `0/1`
  - `request-settlement-finalize-success-rate`: materialized as `0/1`
  - `pricing-lifecycle-synchronize-success-rate`: materialized as `0/1`
- preserved the earlier six truthful closures from the prior loop:
  - `routing-simulation-p95-latency`
  - `api-key-issuance-success-rate`
  - `runtime-rollout-success-rate`
  - `gateway-non-streaming-success-rate`
  - `gateway-streaming-completion-success-rate`
  - `billing-event-write-success-rate`
- wrote the first complete governed latest-artifact chain for the current host:
  - `docs/release/release-telemetry-export-latest.json`
  - `docs/release/release-telemetry-snapshot-latest.json`
  - `docs/release/slo-governance-latest.json`

## Verification

- parser red/green:
  - `node --test --experimental-test-isolation=none scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
- live failover probe:
  - `artifacts/runtime/dev-s08-telemetry-check/release-input/local-governance-fail-replay-live.json`
  - key truth:
    - `gatewayExecutionFailoverLogCount = 1`
    - `primaryFailureSnapshotCount = 1`
    - `backupHealthySnapshotCount = 1`
    - `timeoutSnapshotCount = 0`
- latest-artifact and gate replay:
  - `node scripts/release/slo-governance.mjs --format json`
  - `node scripts/release/run-release-governance-checks.mjs --format json`

## Result

- `release-slo-governance` is no longer `blocked`
- current truthful state is now quantitative `fail`
- passing governed targets now include:
  - `gateway-fallback-success-rate`
  - `gateway-provider-timeout-budget`
  - the six targets closed in the prior derived-target probe
- failing governed targets are now explicit:
  - `admin-api-availability`
  - `account-hold-creation-success-rate`
  - `request-settlement-finalize-success-rate`
  - `pricing-lifecycle-synchronize-success-rate`
- the admin availability miss is now explainable from live `/metrics` rather than from missing evidence:
  - admin counters currently include accumulated `501` responses from unavailable commercial billing routes

## Exit

- Step result: `no-go`
- Reason:
  - release truth is now evidence-complete enough to evaluate
  - the remaining blocker is no longer missing telemetry evidence
  - it is explicit quantitative failure in admin/commercial-control-plane targets plus independent `release-sync-audit` failure
