# 2026-04-08 Step 06 Portal Payment Simulation Posture Hardening Review

## Scope

This review slice continued the active Step 06 commercialization closure lane for the Portal payment boundary.

Execution boundary:

- keep the work inside the currently unlocked Portal commerce compatibility-hardening slice
- close the concrete production-boundary drift between backend posture and frontend billing behavior
- do not claim Step 06, Phase 2, or release closure beyond the evidence collected in this slice

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `production-boundary-hardening`
- Previous mode: `verification-contract-hardening`
- Strategy switch: yes

### Candidate Actions

1. Close the active Portal payment-simulation contract drift by wiring the production posture through the aggregate response, TS types, and billing UI.
   - `Priority Score: 120`
   - `S1` current-step closure push: `5 x 5 = 25`
   - `S2` Step 06 capability / `8.3` / `8.6` push: `4 x 5 = 20`
   - `S3` verification and release-gate push: `5 x 4 = 20`
   - `S4` blocker removal value: `5 x 4 = 20`
   - `S5` commercial delivery push: `4 x 3 = 12`
   - `S6` dual-runtime consistency value: `3 x 3 = 9`
   - `S7` immediate verifiability: `7 x 2 = 14`
   - `P1` churn / rework risk: `0 x -3 = 0`

2. Leave the frontend surface unchanged and rely only on the new backend `409` conflict guard.
   - `Priority Score: 63`
   - rejected because the frontend would keep advertising actions that the production backend intentionally blocks

3. Skip the compatibility posture work and move directly to the larger attempt-backed API migration.
   - `Priority Score: 71`
   - rejected because the currently proven production drift was concrete, user-facing, and cheaper to close now than after a broader API migration

### Chosen Action

Action 1 was selected because it removed an already-proven production-boundary inconsistency with the smallest defensible write surface and immediate testable evidence.

## Root Cause Summary

### 1. Backend and Frontend Disagreed About Payment Simulation Posture

The backend already introduced a production guard:

- `POST /portal/commerce/orders/{order_id}/settle`
- `POST /portal/commerce/orders/{order_id}/payment-events`

When `payment_simulation_enabled=false`, both routes now reject with a `409` conflict.

Result:

- production Portal users were blocked from replay-style settlement at the HTTP layer
- the frontend aggregate model still did not know that this posture existed

### 2. The Aggregate Commerce Contract Did Not Carry the Same Boundary Signal

`checkout-session` already exposed `payment_simulation_enabled`, but `order-center` did not.

Result:

- the billing page could not derive the production posture from the aggregate fetch it already depends on
- portal TS contracts drifted away from the real backend JSON shape

### 3. Billing UI Still Exposed Compatibility Actions in the Production Path

Portal billing still rendered:

- `Settle order`
- provider callback replay actions such as `Simulate provider settlement`

Result:

- the user-facing surface still implied that operator-style payment simulation was part of the production user workflow
- this contradicted the architecture review goal of isolating lab compatibility from the real payment path

## Implemented Fixes

- added `payment_simulation_enabled` to the Portal `order-center` aggregate response
- aligned `PortalCommerceCheckoutSession` and `PortalCommerceOrderCenterResponse` in the portal TypeScript contracts
- added `payment_simulation_enabled` to the billing page repository/types boundary
- wired the billing page state to the aggregate commerce contract through `setPaymentSimulationEnabled(data.payment_simulation_enabled)`
- kept the pending-order cancel path available while hiding manual settlement when payment simulation is disabled
- filtered provider callback simulation actions behind both:
  - `method.supports_webhook`
  - `paymentSimulationEnabled`
- added Rust regression assertions proving:
  - default production Portal order-center posture reports `false`
  - explicit lab/test Portal order-center posture reports `true`

## Files Touched In This Slice

- `crates/sdkwork-api-interface-portal/src/commerce.rs`
- `crates/sdkwork-api-interface-portal/tests/portal_commerce/order_checkout.rs`
- `crates/sdkwork-api-interface-portal/tests/portal_commerce/order_views.rs`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/types/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/repository/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - initially failed on the missing `payment_simulation_enabled: boolean;` contract proof

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `cargo test -p sdkwork-api-interface-portal --test portal_commerce -- --nocapture`
  - verified successfully with:
    - `CARGO_BUILD_JOBS=1`
    - `RUSTFLAGS=-Cdebuginfo=0`

### Observed Constraint

- the default Windows `cargo test` path in the shared `target/debug/deps` directory hit transient MSVC linker PDB failures:
  - `LNK1201`
  - `LNK1136`
- this was an environment/toolchain artifact, not a business-logic regression in the changed Portal code
- the serial no-debuginfo rerun provided the code-level verification evidence for this slice, but the broader Windows Rust verification environment still needs separate stabilization work

## Current Assessment

### Closed In This Slice

- Portal aggregate commerce posture now explicitly reports whether payment simulation is enabled
- portal TS and billing repository/page contracts now match the backend posture boundary
- the default production Portal billing surface no longer advertises manual settlement or provider callback replay as ordinary user actions
- lab/test compatibility posture remains available only through explicit opt-in

### Still Open

- Portal still relies on compatibility-era `order-center`, `checkout-session`, and `payment_events` aggregates instead of a fully attempt-backed formal payment object model
- the legacy compatibility routes still exist and should eventually be removed after attempt-backed detail flows are complete
- pricing truth-source convergence and the broader Phase 2 Portal payment API closure are still open

## Maturity Delta

- `stateful standalone` fact maturity: `L2 -> L3` for the Portal payment-simulation production boundary
- `stateless runtime` fact maturity: unchanged
- Step 06 Portal production-payment boundary maturity: `L2 -> L3`

## Next Slice Recommendation

1. Continue the Portal Phase 2 migration from compatibility aggregates to attempt-backed order/payment-method/detail contracts.
2. Keep the compatibility routes isolated behind explicit lab posture until the formal payment detail APIs are complete.
3. Resume the pricing truth-source closure so Portal checkout/catalog/posture stop mixing `workspace_seed` commerce inputs with canonical pricing evidence.
