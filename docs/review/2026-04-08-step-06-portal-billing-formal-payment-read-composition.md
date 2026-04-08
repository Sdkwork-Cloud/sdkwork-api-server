# 2026-04-08 Step 06 Portal Billing Formal Payment Read Composition Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane after the formal commerce read APIs were published.

Execution boundary:

- keep `GET /portal/commerce/order-center` available as the compatibility event aggregate for now
- stop letting Portal billing treat `order-center` as the only payment-detail truth source
- move the billing repository and payment-history assembly onto explicit composition of formal order detail, payment-method detail, and latest payment-attempt detail

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `portal-display-model-closure`
- Previous mode: `formal-contract-closure`
- Strategy switch: yes

### Candidate Actions

1. Migrate the billing repository to compose formal payment reads while still using compatibility `payment_events` as the current event-evidence bridge.
   - `Priority Score: 132`
   - closes the highest-value frontend coupling without forcing a larger checkout-session redesign

2. Rewrite the entire billing page around attempt-scoped checkout resources first.
   - `Priority Score: 81`
   - rejected because the current runtime still uses order-scoped checkout-session compatibility routes and the page migration would widen scope substantially

3. Leave billing on `order-center` and wait for a later full frontend rewrite.
   - `Priority Score: 34`
   - rejected because it would preserve the known Step 06 architecture gap after the formal read APIs had already shipped

### Chosen Action

Action 1 was selected because it reduces the active architecture drift immediately while preserving a controlled compatibility bridge for payment-event history.

## Root Cause Summary

### 1. Portal Billing Still Used Compatibility Aggregates As The Only Payment Detail Source

The billing repository loaded:

- `GET /portal/commerce/order-center`

and then derived:

- `orders`
- `payment_history`
- `refund_history`
- `membership`
- `commercial_reconciliation`

directly from the aggregate payload.

That meant the newly published formal reads were not improving the actual Portal display model yet.

### 2. Payment History Rows Were Bound To Embedded Aggregate Snapshots

`buildBillingPaymentHistory(...)` used `PortalCommerceOrderCenterEntry` directly, so:

- row target names came from the embedded aggregate order snapshot
- refund/reference posture came from compatibility `checkout_session`
- payment rail labeling had no path to canonical payment-method detail

### 3. The SDK Surface Had Improved, But The Portal Repository Boundary Had Not

The Portal SDK already exposed:

- `getPortalCommerceOrder(orderId)`
- `listPortalCommercePaymentMethods(orderId)`
- `getPortalCommercePaymentAttempt(paymentAttemptId)`

But the billing repository did not call them, so the Step 06 frontend boundary remained effectively aggregate-only.

## Implemented Fixes

- updated `loadBillingPageData()` to:
  - keep the existing billing/account/catalog aggregate reads
  - load formal order detail for each visible billing order
  - load order-scoped payment-method detail for each visible billing order
  - load latest payment-attempt detail where the canonical order points at one
- introduced a `BillingPaymentHistorySource` composition model so payment/refund history rows are now built from:
  - canonical `PortalCommerceOrder`
  - compatibility `payment_events`
  - compatibility `checkout_session`
  - canonical latest `CommercePaymentAttemptRecord`
  - canonical selected `PaymentMethodRecord`
- changed payment/refund history assembly to prefer:
  - canonical order status and target metadata
  - canonical payment-attempt reference when available
  - canonical payment-method display name for audit rows
- updated the billing page payment-rail cell to show the selected payment-method label beneath the provider rail when available
- added and hardened Portal Node source-contract coverage so the repository must keep composing the new formal payment reads

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/repository/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/services/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/types/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed before the repository change because billing still issued only the aggregate reads and never called the formal order/payment-method/payment-attempt endpoints

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal exec tsc --noEmit`

## Current Assessment

### Closed In This Slice

- Portal billing no longer treats `order-center` as the only payment-detail truth source
- the canonical order/payment-method/payment-attempt contract is now consumed by the main billing repository boundary
- the Portal payment/refund history audit views now preserve canonical payment-method naming and latest attempt references where available

### Still Open

- the interactive checkout panel still loads the compatibility `GET /portal/commerce/orders/{order_id}/checkout-session` resource
- provider callback replay and manual settlement remain compatibility-era actions even though they are now hidden behind production posture controls
- pricing truth-source convergence is still open

## Next Slice Recommendation

1. Continue the Portal billing/checkout lane by deciding whether the pending-order detail view should remain order-scoped `checkout-session` or move to an attempt-scoped interactive model.
2. Migrate the next Portal payment-choice or retry surface so it consumes the same formal order/payment-attempt composition instead of reintroducing aggregate-only dependencies elsewhere.
3. Keep the compatibility event bridge temporary: once formal payment-event detail/list reads exist, remove `order-center` from billing-history assembly as well.
