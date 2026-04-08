# 2026-04-07 Step 06 Rust Verification Recovery Review

## Scope

This review slice continued Step 06 in blocker-clearing mode and targeted the real Rust verification blockers that still prevented the control-plane chain from reaching a green service-level state.

Primary target in this round:

- recover `portal-service` and `admin-service` verification through the repository-owned split-package matrix

Execution boundary for this slice:

- fix only concrete compile and manifest regressions on the active Step 06 chain
- do not expand into warning cleanup, feature expansion, or architecture reshaping
- do not claim Step 06 closed while capability, testing, and business-workspace closure remain open

## Decision Ledger

- Date: 2026-04-07
- Version: `v0.1.5`
- Wave / Step: `B / 06`
- Primary mode: `blocker-clearing`
- Previous mode: `blocker-clearing`
- Strategy switch: no

### Candidate Actions

1. Close the concrete Rust compile blockers on the `portal-service` and `admin-service` dependency chain.
   - `Priority Score: 103`
   - `S1` current-step closure push: `5 x 5 = 25`
   - `S2` Step 06 capability / `8.3` / `8.6` push: `4 x 5 = 20`
   - `S3` verification and release-gate push: `5 x 4 = 20`
   - `S4` blocker removal value: `4 x 4 = 16`
   - `S5` commercial delivery push: `2 x 3 = 6`
   - `S6` dual-runtime consistency value: `3 x 3 = 9`
   - `S7` immediate verifiability: `5 x 2 = 10`
   - `P1` churn / rework risk: `1 x -3 = -3`

2. Start warning cleanup in `sdkwork-api-app-gateway` and `sdkwork-api-app-runtime` before the service groups are green.
   - `Priority Score: 54`
   - blocked by lower gate value and weaker Step 06 closure impact

3. Write release and review documents before the service groups are green.
   - `Priority Score: 33`
   - rejected because it would record incomplete facts while the blocker remained active

### Chosen Action

Action 1 was selected because it directly reduced active Step 06 blocker count, produced immediate verification evidence, and avoided widening the write surface before the service chain was green again.

## Root Cause Summary

The blocker pattern was not a new business bug. It was a repeated module-split regression across several crates.

1. `sdkwork-api-interface-portal`
   - crate-root helpers referenced state constants, DTOs, and helper functions that remained private inside split modules
   - `sdkwork-api-app-commerce` exposed `reclaim_expired_coupon_reservations_for_code_if_needed` as a plain internal import instead of a public re-export

2. `sdkwork-api-app-runtime`
   - rollout logic was split across `rollout_execution.rs` and `runtime_reload.rs`, but cross-module helpers, constants, statics, and constructors were not re-exported at the crate root
   - `StandaloneRuntimeSupervision` kept a constructor-relevant field private after the split

3. `portal-api-service`
   - service code imported `sdkwork_api_app_credential::CredentialSecretManager`, but `Cargo.toml` no longer declared `sdkwork-api-app-credential`

4. `sdkwork-api-interface-admin`
   - the interface layer started using `sdkwork-api-app-commerce` without updating its manifest
   - two handlers still called the pre-split commerce signatures and did not pass the now-required secret-manager / billing context

## Implemented Fixes

### Portal Interface Recovery

- promoted `DEFAULT_PORTAL_JWT_SIGNING_SECRET` to `pub(crate)` and re-exported it from the crate root
- re-exported `PortalOrderCenterEntry` and `PortalCommerceReconciliationSummary` for crate-root reconciliation helpers
- promoted the needed `PortalOrderCenterEntry` and `PortalCommerceReconciliationSummary` fields to `pub(crate)`
- promoted `PortalRoutingSummary` to `pub(crate)` so route wiring can legally expose the handler type
- publicly re-exported `reclaim_expired_coupon_reservations_for_code_if_needed` from `sdkwork-api-app-commerce`

### Shared Runtime Rollout Recovery

- promoted the shared rollout timeout constants and rollout ID statics to `pub(crate)`
- promoted the runtime reload helper functions used by `rollout_execution.rs` to `pub(crate)`
- corrected the crate-root internal re-exports so split modules can resolve shared helpers through `use super::*`
- promoted `StandaloneRuntimeSupervision.join_handle` to `pub(crate)` so the runtime reload module can construct the supervision wrapper

### Service / Interface Wiring Recovery

- added missing `sdkwork-api-app-credential` dependency to `services/portal-api-service/Cargo.toml`
- added missing `sdkwork-api-app-commerce` dependency to `crates/sdkwork-api-interface-admin/Cargo.toml`
- updated admin commerce handlers to pass:
  - `state.commercial_billing.as_deref()`
  - `&state.secret_manager`
  to the updated commerce refund / reconciliation APIs

## Files Touched In This Slice

- `crates/sdkwork-api-app-commerce/src/lib.rs`
- `crates/sdkwork-api-app-runtime/src/lib.rs`
- `crates/sdkwork-api-app-runtime/src/rollout_execution.rs`
- `crates/sdkwork-api-app-runtime/src/runtime_core.rs`
- `crates/sdkwork-api-app-runtime/src/runtime_reload.rs`
- `crates/sdkwork-api-interface-admin/Cargo.toml`
- `crates/sdkwork-api-interface-admin/src/commerce.rs`
- `crates/sdkwork-api-interface-portal/src/commerce.rs`
- `crates/sdkwork-api-interface-portal/src/lib.rs`
- `crates/sdkwork-api-interface-portal/src/routing.rs`
- `crates/sdkwork-api-interface-portal/src/state.rs`
- `services/portal-api-service/Cargo.toml`

## Verification Evidence

### Green Commands

- `CARGO_TARGET_DIR=target-check-portal-interface-20260407 cargo check -p sdkwork-api-interface-portal`
- `CARGO_TARGET_DIR=target-check-app-runtime-20260407 cargo check -p sdkwork-api-app-runtime`
- `CARGO_TARGET_DIR=target-check-portal-service-20260407 cargo check -p portal-api-service`
- `CARGO_TARGET_DIR=target-check-portal-service-20260407 node scripts/check-rust-verification-matrix.mjs --group portal-service`
- `CARGO_TARGET_DIR=target-check-admin-service-20260407 cargo check -p sdkwork-api-interface-admin`
- `CARGO_TARGET_DIR=target-check-admin-service-20260407 cargo check -j 1 -p admin-api-service`
- `CARGO_TARGET_DIR=target-check-admin-service-20260407 node scripts/check-rust-verification-matrix.mjs --group admin-service`

### Earlier Prerequisites Still Valid

The following gates were already green before this slice and were not invalidated by the current minimal fixes:

- `cargo check -p sdkwork-api-app-gateway`
- `cargo check -p sdkwork-api-storage-postgres`
- `cargo check -p sdkwork-api-extension-host`
- `cargo check -p sdkwork-api-provider-openai`
- `cargo check -p sdkwork-api-app-routing`
- `cargo check -p sdkwork-api-app-commerce`

## Current Assessment

### Closed In This Slice

- the Step 06 Rust verification blocker on the portal control-plane chain is closed
- the Step 06 Rust verification blocker on the admin control-plane chain is closed
- the remaining service-level blocker profile moved from hard compile failures to warning debt

### Still Open

- Step 06 is not yet complete
- architecture writeback for full Step 06 capability closure is not done in this slice
- warning debt remains in `sdkwork-api-app-gateway`, `sdkwork-api-app-runtime`, `sdkwork-api-app-commerce`, `sdkwork-api-storage-sqlite`, and `sdkwork-api-config`
- broader Step 06 closure still needs business-workspace, E2E, scoring, and capability-to-doc closure work

## Maturity Delta

- `stateful standalone` fact maturity: `L2 -> L3`
  - service boot chains for admin and portal now compile through the Step 06 matrix again
- `stateless runtime` fact maturity: unchanged at `L2`
  - this slice did not add new stateless proof assets

## Recommended Next Slice

1. Move Step 06 from blocker-clearing into verification-solidification for the recovered admin / portal commerce surfaces.
2. Add or refresh high-value tests around:
   - admin commerce refund and reconciliation handlers
   - portal order-center and reconciliation summary paths
   - runtime rollout supervision wiring touched by the split-module recovery
3. Re-evaluate Step 06 `8.3` / `8.6` closure against the actual codebase and only then decide whether architecture writeback can be marked complete.
