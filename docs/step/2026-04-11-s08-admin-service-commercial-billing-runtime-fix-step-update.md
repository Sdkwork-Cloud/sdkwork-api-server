# 2026-04-11 S08 Admin Service Commercial Billing Runtime Fix Step Update

## Scope

- Step: `S08 integrated acceptance, release gate, and continuous iteration`
- Loop target: close the repo-local root cause behind the admin commercial-control-plane `501` responses and raise the evidence from route-shape verification to runtime-and-metrics verification
- Boundaries:
  - `services/admin-api-service/src/main.rs`
  - `services/admin-api-service/Cargo.toml`
  - `docs/step/110-S08-集成验收-发布门禁与持续迭代-2026-04-10.md`
  - `docs/架构/166-API产品-商业账户-coupon-first营销统一架构-2026-04-10.md`
  - `docs/review/2026-04-11-commercial-s08-admin-service-commercial-billing-runtime-fix-review.md`
  - `docs/release/2026-04-11-v0.1.50-commercial-s08-admin-service-commercial-billing-runtime-fix.md`

## Root Cause Investigation

- the prior `501` posture was not caused by a missing billing kernel implementation in the repo
- root cause:
  - `services/admin-api-service/src/main.rs` assembled only the admin store and omitted the commercial billing kernel from the standalone runtime state
  - portal/product runtime paths already proved the intended bootstrap pattern
- current session constraint:
  - host policy blocks the child-process/background launch pattern needed to replay the full source-backed stack from this shell
  - because of that, this loop upgrades the evidence through the service runtime itself rather than by claiming a new live latest-artifact replay

## Changes

- switched admin standalone bootstrap from the store-only path to `build_admin_store_and_commercial_billing_from_config(...)`
- introduced `build_admin_service_runtime(config)` so the tested runtime path and the production startup path are now the same code path
- added a service-level regression test that verifies:
  - login succeeds
  - the three commercial-control-plane routes succeed
  - metrics record `200` counters for those routes
  - metrics do not record `501` counters for those routes

## Verification

- targeted service metrics regression:
  - `cargo test -p admin-api-service build_admin_service_runtime_reports_success_metrics_for_commercial_billing_routes -- --nocapture`
- full package verification:
  - `cargo test -p admin-api-service -- --nocapture`

## Result

- the repo-local root cause is now closed
- the current source runtime no longer reproduces the old `501` commercial-control-plane path in service-level verification
- governed live latest artifacts were not refreshed on this host in this loop, so the global `S08` release gate cannot yet be upgraded from the last live replay result

## Exit

- Step result: `conditional-go`
- Reason:
  - code and service-runtime truth improved materially and is now covered by regression tests
  - a fresh governed host-level replay is still required before `S08` can change its overall `no-go` posture
