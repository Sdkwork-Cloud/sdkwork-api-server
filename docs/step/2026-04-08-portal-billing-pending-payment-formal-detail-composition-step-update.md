# 2026-04-08 Portal Billing Pending Payment Formal Detail Composition Step Update

## Slice Goal

Close the next Portal Step 06 payment-display gap by moving the pending-order detail panel to a formal-first payment read composition while keeping the current compatibility checkout-session workbench available.

## Closed In This Slice

- added `getBillingCheckoutDetail(orderId)` to the Portal billing repository
- pending-order detail now composes:
  - `GET /portal/commerce/orders/{order_id}`
  - `GET /portal/commerce/orders/{order_id}/payment-methods`
  - `GET /portal/commerce/payment-attempts/{payment_attempt_id}`
  - `GET /portal/commerce/orders/{order_id}/checkout-session`
- the Portal billing `Checkout session` panel now prefers canonical latest-attempt reference and canonical selected payment-method identity in its top detail facts

## Runtime / Display Truth

### Formal-First Pending Payment Detail

- canonical order detail is loaded first
- canonical available payment methods are loaded first
- canonical latest payment attempt is loaded first when available
- compatibility checkout-session remains the interaction rail, not the sole data truth

### Compatibility Bridge Still In Use

- checkout-session method list
- callback rehearsal / simulated provider settlement flows
- lab-style operator settlement bridge

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal exec tsc --noEmit`

## Remaining Follow-Up

1. Replace compatibility checkout-session method cards with a canonical payment-method / latest-attempt driven action model.
2. Keep shrinking user-facing compatibility payment actions so Portal can eventually treat checkout-session as a temporary bridge only.
3. Continue the broader Step 06 payment closure until retry, payment-choice, and final interactive flows are all formal-model-backed.
