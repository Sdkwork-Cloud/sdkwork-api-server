# 2026-04-11 Commercial S08 Governance Fail Replay Review

## Scope

- Architecture reference: `166`
- Step reference: `110`
- Loop focus: verify whether the current host can now produce a complete governed release telemetry chain and, if so, whether `S08` exits as `go`, `fail`, or still `blocked`

## Findings

### P0 - the remaining S08 release telemetry lane is no longer blocked; it now fails quantitatively

- a live local stateful-provider probe now proves the current runtime can emit truthful fallback evidence:
  - `RoutingDecisionLog` with `fallback_reason = gateway_execution_failover`
  - `ProviderHealthSnapshot` for both the failed primary and the healthy backup
  - gateway metrics:
    - `sdkwork_upstream_requests_total`
    - `sdkwork_gateway_failovers_total`
    - `sdkwork_provider_health_status`
- that evidence closes the two previously-missing gateway targets:
  - `gateway-fallback-success-rate = 1`
  - `gateway-provider-timeout-budget = 0`
- impact:
  - `release-slo-governance` has now moved from `blocked` to `fail`
  - the gate posture is materially stronger because it is now evidence-backed rather than missing-input-backed

### P0 - snapshot materialization needed a parser bug fix before truthful live artifacts could exist

- direct live replay initially failed even after the fallback probe succeeded
- root cause:
  - `materialize-release-telemetry-snapshot.mjs` split Prometheus sample lines with a regex that terminated on the first `}` character
  - current admin `/metrics` includes route labels like `/admin/runtime-config/rollouts/{rollout_id}`
  - this made truthful export replay crash on valid live metrics text
- fix:
  - added a failing test to `scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
  - replaced the sample-line splitter with quote-aware scanning so braces inside label values no longer corrupt parsing
  - reran the snapshot suite to green
- impact:
  - governed latest artifacts can now be materialized from the current live admin metrics

### P1 - the commercial billing control plane is now quantitatively failing instead of merely described as unavailable

- truthful live endpoint replay now materializes these targets as failures:
  - `account-hold-creation-success-rate = 0`
  - `request-settlement-finalize-success-rate = 0`
  - `pricing-lifecycle-synchronize-success-rate = 0`
- direct evidence:
  - `GET /admin/billing/account-holds`: `501`
  - `GET /admin/billing/request-settlements`: `501`
  - `POST /admin/billing/pricing-lifecycle/synchronize`: `501`
- impact:
  - the release gate now records these as explicit quantitative failures
  - the remaining blocker is runtime capability absence, not documentation uncertainty

### P1 - admin availability is also failing because the unavailable commercial routes are now visible in the governed metrics

- `admin-api-availability` now evaluates to `0.935897`, below the `0.999` objective
- direct admin HTTP totals currently aggregate:
  - `200`: `112`
  - `201`: `32`
  - `401`: `1`
  - `404`: `1`
  - `501`: `10`
- impact:
  - admin availability failure is not a separate mystery
  - it is materially driven by repeated `501` responses on the unavailable commercial-control-plane routes

## Fix Closure

- corrected the release telemetry materializer so current live admin metrics can be parsed truthfully
- materialized the full governed latest-artifact chain under `docs/release/*latest.json`
- converted the honest `S08` release posture from:
  - `window pass / sync fail / SLO blocked`
- into:
  - `window pass / sync fail / SLO fail`

## Verification

- TDD:
  - `node --test --experimental-test-isolation=none scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs`
- live governed replay:
  - `node scripts/release/slo-governance.mjs --format json`
  - `node scripts/release/run-release-governance-checks.mjs --format json`
- evidence files:
  - `artifacts/runtime/dev-s08-telemetry-check/release-input/local-governance-fail-replay-live.json`
  - `artifacts/runtime/dev-s08-telemetry-check/release-input/supplemental-targets-local-governance-fail-replay.json`
  - `docs/release/release-telemetry-snapshot-latest.json`
  - `docs/release/slo-governance-latest.json`

## Residual Risks

- `release-sync-audit` remains independently failing even after the SLO lane is fully materialized
- the portal billing account route still remains outside the quantitative baseline and still does not indicate a provisioned live commercial account
- later `S08` doc backwrite changes the workspace size, so `release-window-snapshot-latest.json` must be refreshed after this loop closes

## Exit

- Step result: `no-go`
- Reason:
  - the current release posture is no longer blocked by missing telemetry evidence
  - it remains a release `no-go` because the governed SLO lane now fails quantitatively and release sync hygiene still fails independently
