# 2026-04-11 S08 Local Derived Target Probe Step Update

## Scope

- Step: `S08 integrated acceptance, release gate, and continuous iteration`
- Loop target: prove which remaining non-availability SLO targets can already be truthfully derived from the live local runtime, then re-locate the first governed snapshot failure after those targets are backfilled
- Boundaries:
  - `artifacts/runtime/dev-s08-telemetry-check/release-input/local-derived-target-probe-live.json`
  - `artifacts/runtime/dev-s08-telemetry-check/release-input/supplemental-targets-local-probe.json`
  - `artifacts/runtime/dev-s08-telemetry-check/release-input/release-telemetry-export-local-probe.json`
  - `docs/step/110-S08-集成验收-发布门禁与持续迭代-2026-04-10.md`
  - `docs/review/*`
  - `docs/release/*`
  - `docs/架构/166-*`

## Root Cause Investigation

- the live local probe now proves six previously-missing non-availability targets are already derivable from truthful runtime actions:
  - `routing-simulation-p95-latency`: `20/20` successful admin route simulations, `p95Ms = 30`
  - `api-key-issuance-success-rate`: `3/3`
  - `runtime-rollout-success-rate`: `1/1`, rollout status reached `succeeded`
  - `gateway-non-streaming-success-rate`: `3/3`
  - `gateway-streaming-completion-success-rate`: `3/3`, every stream completed with `[DONE]`
  - `billing-event-write-success-rate`: `6/6`
- correlated probe evidence confirms those values were observed from the active runtime rather than invented:
  - the gateway probe API key produced six usage records
  - the same probe produced six billing events
  - live gateway routing decision logs recorded `decision_source = gateway`
  - billing events and decision logs carried `fallback_reason = policy_candidate_unavailable`
- the truthful blocker is now narrower than "missing supplemental targets":
  - gateway `/metrics` still emits no `sdkwork_upstream_requests_total` samples
  - gateway `/metrics` still emits no `sdkwork_upstream_retries_total` samples
  - gateway `/metrics` still emits no `sdkwork_gateway_failovers_total` samples
  - gateway `/metrics` still emits no `sdkwork_gateway_execution_context_failures_total` samples
- canonical commercial control-plane evidence is also still incomplete on the current runtime:
  - `GET /admin/billing/account-holds`: `501`, `commercial billing control plane is unavailable for the current storage runtime`
  - `GET /admin/billing/request-settlements`: `501`, same runtime-unavailable message
  - `POST /admin/billing/pricing-lifecycle/synchronize`: `501`, same runtime-unavailable message
  - `GET /portal/billing/account`: `404`, `workspace commercial account is not provisioned`
- governed snapshot replay confirms the blocker shift precisely:
  - `release-telemetry-export-local-probe.json` materializes successfully from live managed metrics plus truthful supplemental targets for the six probeable metrics
  - `materialize-release-telemetry-snapshot.mjs` now fails at `gateway-fallback-success-rate`

## Changes

- converted the remaining S08 observability blocker from "truthful supplemental targets are broadly absent" into a narrower statement backed by live probe evidence
- documented that six non-availability targets are already closeable from real local runtime operations
- documented that the first missing governed target has advanced from `gateway-non-streaming-success-rate` to `gateway-fallback-success-rate`
- recorded the remaining truthful blockers as:
  - fallback / provider-timeout evidence on provider-level gateway metrics
  - canonical commercial control-plane kernel unavailability for holds, settlements, and pricing sync
  - portal commercial account surface still not provisioned on the current live runtime
  - independent `release-sync-audit` failure

## Verification

- live probe artifacts:
  - `Get-Content artifacts/runtime/dev-s08-telemetry-check/release-input/local-derived-target-probe-live.json`
  - `Get-Content artifacts/runtime/dev-s08-telemetry-check/release-input/supplemental-targets-local-probe.json`
  - `Get-Content artifacts/runtime/dev-s08-telemetry-check/release-input/release-telemetry-export-local-probe.json`
- governed snapshot replay:
  - `node scripts/release/materialize-release-telemetry-snapshot.mjs --export artifacts/runtime/dev-s08-telemetry-check/release-input/release-telemetry-export-local-probe.json --output artifacts/runtime/dev-s08-telemetry-check/release-input/release-telemetry-snapshot-local-probe.json`
  - current failure point: `release telemetry snapshot is missing target gateway-fallback-success-rate`

## Result

- `release-slo-governance` is still blocked on current repo truth
- but the blocked surface is now materially smaller:
  - six non-availability targets are truthfully probeable locally
  - the first remaining governed snapshot gap is now `gateway-fallback-success-rate`
  - the remaining commercial-control-plane gaps are explicit endpoint/runtime availability gaps rather than generic uncertainty
- `release-window-snapshot` remains `pass`
- `release-sync-audit` remains `fail`

## Exit

- Step result: `no-go`
- Reason:
  - `S08` is no longer blocked by the first six non-availability targets
  - it remains blocked by missing truthful fallback / timeout evidence, unavailable commercial control-plane kernel routes, and independent release-sync hygiene failure
