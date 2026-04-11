# 2026-04-11 Commercial S08 Local Derived Target Probe Review

## Scope

- Architecture reference: `166`
- Step reference: `110`
- Loop focus: verify how far live local runtime probes can truthfully close the remaining S08 non-availability SLO baseline and identify the next exact governed blocker

## Findings

### P0 - six previously-missing non-availability targets are now truthfully closed, and the first missing governed target has moved to fallback

- the live probe evidence pack now truthfully derives:
  - `routing-simulation-p95-latency`: `20/20`, `p95Ms = 30`
  - `api-key-issuance-success-rate`: `3/3`
  - `runtime-rollout-success-rate`: `1/1`
  - `gateway-non-streaming-success-rate`: `3/3`
  - `gateway-streaming-completion-success-rate`: `3/3`
  - `billing-event-write-success-rate`: `6/6`
- when those six targets are materialized into `supplemental-targets-local-probe.json` and replayed through the governed export chain, snapshot derivation no longer fails at `gateway-non-streaming-success-rate`
- the next exact failure is now:
  - `release telemetry snapshot is missing target gateway-fallback-success-rate`
- impact:
  - the remaining S08 telemetry blocker is materially smaller and more precise
  - truthful local probe evidence already closes a meaningful subset of the governed SLO baseline

### P1 - gateway fallback and timeout coverage are still blocked by missing provider-level execution samples, not by lack of gateway traffic

- the probe generated real gateway traffic and real downstream side effects:
  - usage records were created
  - billing events were created
  - routing decision logs were created with `decision_source = gateway`
  - those records also carried `fallback_reason = policy_candidate_unavailable`
- despite that, gateway `/metrics` still emitted no provider-level samples for:
  - `sdkwork_upstream_requests_total`
  - `sdkwork_upstream_retries_total`
  - `sdkwork_gateway_failovers_total`
  - `sdkwork_gateway_execution_context_failures_total`
- impact:
  - fallback / timeout SLO targets remain blocked because the live runtime never emitted truthful failover, timeout, or retry evidence
  - the problem statement is now "missing provider-level execution evidence", not "no runtime evidence exists"

### P1 - canonical commercial control-plane routes remain unavailable on the current runtime

- current live endpoint truth:
  - `GET /admin/billing/account-holds`: `501`
  - `GET /admin/billing/request-settlements`: `501`
  - `POST /admin/billing/pricing-lifecycle/synchronize`: `501`
  - `GET /portal/billing/account`: `404`
- returned messages are explicit:
  - admin billing kernel routes: `commercial billing control plane is unavailable for the current storage runtime`
  - portal account route: `workspace commercial account is not provisioned`
- impact:
  - `account-hold-creation-success-rate`, `request-settlement-finalize-success-rate`, and `pricing-lifecycle-synchronize-success-rate` remain blocked by runtime capability absence, not by missing documentation
  - portal commercial account truth is still not provisioned for live acceptance closure

## Fix Closure

- corrected S08 documentation from a broad "supplemental targets missing" statement to an evidence-backed split:
  - six targets are already truthfully derivable locally
  - fallback / timeout and commercial control-plane targets remain blocked
- backwrote `110`, `166`, changelog, and the new release note to preserve the same narrower blocker wording
- kept the final commercialization posture honest at `no-go`

## Verification

- evidence pack inspection:
  - `Get-Content artifacts/runtime/dev-s08-telemetry-check/release-input/local-derived-target-probe-live.json`
  - `Get-Content artifacts/runtime/dev-s08-telemetry-check/release-input/supplemental-targets-local-probe.json`
  - `Get-Content artifacts/runtime/dev-s08-telemetry-check/release-input/release-telemetry-export-local-probe.json`
- governed snapshot replay:
  - `node scripts/release/materialize-release-telemetry-snapshot.mjs --export artifacts/runtime/dev-s08-telemetry-check/release-input/release-telemetry-export-local-probe.json --output artifacts/runtime/dev-s08-telemetry-check/release-input/release-telemetry-snapshot-local-probe.json`

## Residual Risks

- default governed `release-telemetry-snapshot-latest.json` and `slo-governance-latest.json` are still not materialized into `docs/release`
- `release-sync-audit` remains independently negative even if telemetry later closes
- truthful fallback / timeout evidence still requires provider-level execution samples instead of inferred placeholders

## Exit

- Step result: `no-go`
- Reason:
  - current loop closed six SLO targets truthfully
  - remaining release truth still lacks fallback / timeout telemetry evidence and commercial control-plane runtime closure
