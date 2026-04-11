# 2026-04-11 Commercial S08 Admin Service Commercial Billing Runtime Fix Review

## Scope

- Architecture reference: `166`
- Step reference: `110`
- Loop focus: determine whether the remaining admin commercial-control-plane `501` failures were a real repo-local runtime bug and, if so, close that bug with stronger source-level evidence

## Findings

### P0 - the standalone admin service was the runtime outlier, and that bootstrap gap is now closed

- `services/admin-api-service/src/main.rs` previously assembled only the admin store and omitted the commercial billing kernel
- the working source patterns already existed in:
  - `services/portal-api-service/src/main.rs`
  - `crates/sdkwork-api-product-runtime/src/lib.rs`
- fix:
  - switched admin standalone bootstrap to `build_admin_store_and_commercial_billing_from_config(...)`
  - threaded the resulting kernel into `AdminApiState`
  - threaded the same kernel into `StandaloneServiceReloadHandles`
- impact:
  - the current source runtime no longer reproduces the old "commercial billing control plane is unavailable for the current storage runtime" posture

### P0 - service-level metrics now prove the three governed commercial routes run on the `200` path instead of the `501` path

- new regression coverage logs in through the real admin auth route, exercises the three commercial-control-plane routes, and then inspects `/metrics`
- exact verified route set:
  - `GET /admin/billing/account-holds`
  - `GET /admin/billing/request-settlements`
  - `POST /admin/billing/pricing-lifecycle/synchronize`
- exact verified metrics posture:
  - each matching `sdkwork_http_requests_total{...,status="200"}` counter increments by `1`
  - each matching `sdkwork_http_requests_total{...,status="501"}` counter remains `0`
- impact:
  - the previous governed commercial failure reason is now closed at the source-service layer

### P1 - the release-governance latest artifacts are still stale relative to this fix

- the current session could not replay a source-backed live stack on this host because background child-process launch is blocked by shell policy
- impact:
  - the last governed latest artifacts under `docs/release/*latest.json` still represent the pre-fix live replay
  - `S08` cannot yet claim the post-fix live gate has improved, even though the repo-local runtime bug is fixed

## Fix Closure

- closed the repo-local admin standalone runtime wiring defect
- strengthened the service evidence from "route exists" to "route succeeds and metrics confirm the success path"
- preserved honest `S08` status boundaries by not rewriting governed live latest artifacts without a fresh replay

## Verification

- targeted regression:
  - `cargo test -p admin-api-service build_admin_service_runtime_reports_success_metrics_for_commercial_billing_routes -- --nocapture`
- package verification:
  - `cargo test -p admin-api-service -- --nocapture`

## Residual Risks

- the next host-level replay still needs to prove that the refreshed live metrics close:
  - `admin-api-availability`
  - `account-hold-creation-success-rate`
  - `request-settlement-finalize-success-rate`
  - `pricing-lifecycle-synchronize-success-rate`
- `release-sync-audit` remains independently failing on the last governed replay
- after the next doc backwrite, `release-window-snapshot-latest.json` must be refreshed again before any final gate claim

## Exit

- Step result: `conditional-go`
- Reason:
  - the repo-local runtime defect is fixed and verified
  - the overall `S08` release gate remains `no-go` until a fresh host-level governed replay updates the live latest artifacts
