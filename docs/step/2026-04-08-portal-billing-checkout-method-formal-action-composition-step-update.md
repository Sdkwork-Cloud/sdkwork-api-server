# 2026-04-08 Portal Billing Checkout Method Formal Action Composition Step Update

## Slice Goal

Close the next Portal Step 06 payment-display gap by moving checkout method cards and callback-rail selection onto a formal-first billing checkout method model.

## Closed In This Slice

- added `buildBillingCheckoutMethods(...)` in Portal billing services
- extended `BillingCheckoutDetail` with `checkout_methods`
- `getBillingCheckoutDetail(orderId)` now returns a normalized checkout method list built from:
  - `GET /portal/commerce/orders/{order_id}`
  - `GET /portal/commerce/orders/{order_id}/payment-methods`
  - `GET /portal/commerce/payment-attempts/{payment_attempt_id}`
  - `GET /portal/commerce/orders/{order_id}/checkout-session`
- the Portal billing page now renders:
  - checkout method cards from `checkoutDetail.checkout_methods`
  - provider callback rehearsal rails from `checkoutDetail.checkout_methods`
  - compatibility `checkoutSession.methods` only as a fallback bridge

## Runtime / Display Truth

### Formal-First Checkout Method Identity

- canonical payment methods define provider rail identity first
- canonical latest payment attempt defines the active provider reference first
- compatibility checkout-session still contributes operator bridge actions and fallback metadata

### Compatibility Bridge Still In Use

- order-scoped compatibility checkout-session payload
- lab/operator settlement bridge
- compatibility payment-event simulation mutation path

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal exec tsc --noEmit`

## Remaining Follow-Up

1. Move retry and provider-launch interactions onto explicit payment-attempt flows instead of compatibility bridges.
2. Reduce remaining order-scoped compatibility settlement actions to a temporary operator-only bridge.
3. Continue the broader Step 06 payment closure until the Portal checkout workbench is fully attempt-backed.
