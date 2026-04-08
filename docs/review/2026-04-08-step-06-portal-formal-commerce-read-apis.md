# 2026-04-08 Step 06 Portal Formal Commerce Read APIs Review

## Scope

This review slice continued the Step 06 Portal commercialization closure lane by addressing the next concrete payment-contract gap after production posture hardening.

Execution boundary:

- keep the compatibility `order-center` and order-scoped `checkout-session` surface available for now
- close the missing formal read/detail APIs that architecture documents already require
- align runtime routes, OpenAPI, SDK, and regression coverage before moving to a larger attempt-backed UI migration

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `formal-contract-closure`
- Previous mode: `production-boundary-hardening`
- Strategy switch: yes

### Candidate Actions

1. Close the missing Portal formal read/detail APIs first, without widening scope to a larger checkout-session redesign.
   - `Priority Score: 124`
   - highest immediate architecture truth value with the smallest write surface

2. Skip directly to an attempt-scoped hosted-checkout route.
   - `Priority Score: 76`
   - rejected because the current documented gap already included three simpler missing read/detail contracts that blocked runtime/schema/SDK alignment

3. Keep relying on `order-center` and defer formal detail APIs.
   - `Priority Score: 41`
   - rejected because this would preserve a known architecture/runtime drift and keep the frontend dependent on compatibility aggregates

### Chosen Action

Action 1 was selected because it closes real missing contracts immediately while preserving flexibility on the later attempt-scoped checkout-session design.

## Root Cause Summary

### 1. Architecture Required Formal Detail APIs That Runtime Did Not Expose

The Portal architecture and step audit already required:

- `GET /portal/commerce/orders/{order_id}`
- `GET /portal/commerce/orders/{order_id}/payment-methods`
- `GET /portal/commerce/payment-attempts/{payment_attempt_id}`

The runtime router only exposed:

- `GET /portal/commerce/order-center`
- `GET /portal/commerce/orders/{order_id}/payment-attempts`
- `GET /portal/commerce/orders/{order_id}/checkout-session`

### 2. Application-Layer Read Helpers Existed Only Partially

The commerce app layer already had:

- workspace/user-safe order loading
- order-scoped payment attempt listing
- checkout-method filtering logic

But it did not expose:

- a public canonical order-detail loader
- a public single payment-attempt loader with workspace ownership checks
- a public filtered payment-method list for one order

### 3. OpenAPI and Portal SDK Were Still Publishing the Old Compatibility Surface

Portal OpenAPI previously published only the aggregate `order-center` route in the commerce payment area.

Portal TypeScript SDK also lacked caller methods for the three formal detail reads.

Result:

- runtime, schema, SDK, and architecture documents were not describing the same payment detail contract
- downstream Portal pages could not migrate safely toward an attempt-backed model

## Implemented Fixes

- exposed `load_portal_commerce_order(...)` from the commerce app layer
- exposed `list_portal_commerce_payment_methods(...)` from the commerce app layer using the existing order-compatibility filter rules
- exposed `load_portal_commerce_payment_attempt(...)` from the commerce app layer with workspace/user ownership checks
- added Portal HTTP handlers for:
  - `GET /portal/commerce/orders/{order_id}`
  - `GET /portal/commerce/orders/{order_id}/payment-methods`
  - `GET /portal/commerce/payment-attempts/{payment_attempt_id}`
- registered those routes in both the placeholder inventory router and the real stateful Portal router
- updated the static Portal OpenAPI document with the new paths and schemas:
  - `PortalCommerceOrder`
  - `PaymentMethodRecord`
  - `CommercePaymentAttemptRecord`
- updated the Portal TypeScript contracts and SDK methods:
  - `getPortalCommerceOrder(orderId)`
  - `listPortalCommercePaymentMethods(orderId)`
  - `getPortalCommercePaymentAttempt(paymentAttemptId)`

## Files Touched In This Slice

- `crates/sdkwork-api-app-commerce/src/lib.rs`
- `crates/sdkwork-api-app-commerce/src/order.rs`
- `crates/sdkwork-api-app-commerce/src/payment_attempt.rs`
- `crates/sdkwork-api-app-commerce/src/payment_method.rs`
- `crates/sdkwork-api-interface-portal/src/commerce.rs`
- `crates/sdkwork-api-interface-portal/src/lib.rs`
- `crates/sdkwork-api-interface-portal/src/openapi.rs`
- `crates/sdkwork-api-interface-portal/tests/openapi_route.rs`
- `crates/sdkwork-api-interface-portal/tests/portal_commerce/mod.rs`
- `crates/sdkwork-api-interface-portal/tests/portal_commerce/order_views.rs`
- `crates/sdkwork-api-interface-portal/tests/portal_commerce/support.rs`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts`
- `apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`

## Verification Evidence

### Red First

- `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_order_detail_returns_canonical_order_view -- --nocapture`
  - failed with `404` before the new `GET /portal/commerce/orders/{order_id}` route existed
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed with `portalApi.getPortalCommerceOrder is not a function` before the new SDK methods existed

### Green

- `cargo test -p sdkwork-api-interface-portal --test portal_commerce -- --nocapture`
  - verified successfully with:
    - `CARGO_BUILD_JOBS=1`
    - `RUSTFLAGS=-Cdebuginfo=0`
- `cargo test -p sdkwork-api-interface-portal --test openapi_route -- --nocapture`
  - verified successfully with:
    - `CARGO_BUILD_JOBS=1`
    - `RUSTFLAGS=-Cdebuginfo=0`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`

## Current Assessment

### Closed In This Slice

- Portal now exposes formal order detail, available payment-method detail, and payment-attempt detail APIs
- OpenAPI and Portal SDK now publish the same formal detail routes as the runtime
- compatibility aggregates remain available, but no longer block consumers from using canonical detail APIs

### Still Open

- attempt-scoped hosted-checkout/session creation is still not a separate route
- the Portal display model still depends heavily on `order-center` and order-scoped `checkout-session`
- pricing truth-source convergence is still open

## Maturity Delta

- Portal formal payment detail contract maturity: `L1 -> L3`
- Portal runtime/schema/SDK alignment for payment detail reads: `L1 -> L3`

## Next Slice Recommendation

1. Decide whether the architecture will keep `create-attempt` returning `checkout_url` as the formal hosted-checkout contract, or add a dedicated attempt-scoped checkout-session route.
2. Start migrating Portal display/retry/payment-choice pages off `order-center` and onto order detail + payment methods + attempt detail composition.
3. Continue the pricing truth-source closure so payment surfaces stop mixing compatibility data with seeded pricing inputs.
