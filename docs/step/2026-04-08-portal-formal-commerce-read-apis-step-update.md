# 2026-04-08 Portal Formal Commerce Read APIs Step Update

## Slice Goal

Close the next Portal payment-contract gap identified in the Step 06 transaction-closure audit without widening scope into a larger hosted-checkout redesign.

## Closed In This Slice

- shipped `GET /portal/commerce/orders/{order_id}`
- shipped `GET /portal/commerce/orders/{order_id}/payment-methods`
- shipped `GET /portal/commerce/payment-attempts/{payment_attempt_id}`
- aligned Portal runtime routes, OpenAPI, SDK, and TypeScript types for those formal read/detail APIs
- kept workspace/user ownership checks inside the commerce app layer instead of re-implementing them at the HTTP edge

## Runtime / Contract Truth

### Now Formal And Published

- `GET /portal/commerce/orders/{order_id}`
- `GET /portal/commerce/orders/{order_id}/payment-methods`
- `GET /portal/commerce/payment-attempts/{payment_attempt_id}`

### Still Compatibility-Oriented

- `GET /portal/commerce/order-center`
- `GET /portal/commerce/orders/{order_id}/checkout-session`
- `POST /portal/commerce/orders/{order_id}/payment-attempts`
  - current formal hosted-checkout behavior remains: create-attempt directly returns `checkout_url`

### Still Not Added As A Separate Route

- `POST /portal/commerce/payment-attempts/{payment_attempt_id}/stripe/checkout-session`
  - current decision: do not add this route yet
  - current contract: `create-attempt` returning `checkout_url` is the formal hosted-checkout entry point unless architecture later requires a dedicated attempt-scoped checkout-session resource

## Verification

- `cargo test -p sdkwork-api-interface-portal --test portal_commerce -- --nocapture`
  - verified with `CARGO_BUILD_JOBS=1` and `RUSTFLAGS=-Cdebuginfo=0`
- `cargo test -p sdkwork-api-interface-portal --test openapi_route -- --nocapture`
  - verified with `CARGO_BUILD_JOBS=1` and `RUSTFLAGS=-Cdebuginfo=0`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`

## Remaining Follow-Up

1. Decide whether the architecture should keep `create-attempt -> checkout_url` as the long-term formal hosted-checkout contract or introduce a dedicated attempt-scoped checkout-session route.
2. Move Portal payment-result, retry, and payment-choice views away from `order-center` aggregation and toward explicit composition of order detail + payment methods + attempt detail.
3. Continue pricing truth-source convergence so payment detail pages stop mixing canonical payment reads with compatibility/seeded pricing inputs.
