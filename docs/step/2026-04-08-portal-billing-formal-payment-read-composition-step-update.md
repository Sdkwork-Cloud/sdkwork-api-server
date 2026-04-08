# 2026-04-08 Portal Billing Formal Payment Read Composition Step Update

## Slice Goal

Close the next Step 06 Portal frontend coupling by moving the billing repository off aggregate-only payment detail reads while preserving the current compatibility event bridge.

## Closed In This Slice

- Portal billing now composes:
  - `GET /portal/commerce/orders/{order_id}`
  - `GET /portal/commerce/orders/{order_id}/payment-methods`
  - `GET /portal/commerce/payment-attempts/{payment_attempt_id}`
- `order-center` remains in place only for:
  - compatibility `payment_events`
  - compatibility `checkout_session`
  - membership / reconciliation aggregate posture
- billing payment/refund history rows now prefer canonical order detail, canonical latest attempt reference, and canonical payment-method naming where available

## Runtime / Display Truth

### Billing Repository Now Uses Formal Payment Reads

- `getPortalCommerceOrder(orderId)`
- `listPortalCommercePaymentMethods(orderId)`
- `getPortalCommercePaymentAttempt(paymentAttemptId)`

### Compatibility Bridge Still In Use

- `getPortalCommerceOrderCenter()`
  - current role: event-evidence bridge plus aggregate membership/reconciliation posture
- `getPortalCommerceCheckoutSession(orderId)`
  - current role: pending-order interactive payment rail view

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal exec tsc --noEmit`

## Remaining Follow-Up

1. Keep migrating pending-order, retry, and payment-choice interactions away from compatibility `checkout-session` and toward a cleaner attempt-backed display model.
2. Replace the current compatibility `payment_events` bridge once a formal Portal payment-event detail/list contract exists.
3. Continue pricing truth-source convergence so Portal billing no longer mixes formal payment reads with seeded purchase catalog behavior.
