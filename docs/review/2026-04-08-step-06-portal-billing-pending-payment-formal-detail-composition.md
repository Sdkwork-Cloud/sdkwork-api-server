# 2026-04-08 Step 06 Portal Billing Pending Payment Formal Detail Composition Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane after billing history had already moved onto formal order/payment reads.

Execution boundary:

- keep the existing Portal `Checkout session` workbench visible
- stop treating compatibility `GET /portal/commerce/orders/{order_id}/checkout-session` as the only source for pending-order detail
- compose canonical order, payment-method, and latest payment-attempt reads first, then use checkout-session only as the compatibility interactive rail

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `pending-payment-detail-closure`
- Previous mode: `portal-display-model-closure`
- Strategy switch: yes

### Candidate Actions

1. Add a repository-level checkout-detail composition helper and route the existing billing panel through it.
   - `Priority Score: 136`
   - highest architecture value for the smallest frontend write surface

2. Redesign the entire pending-payment UI around a brand-new attempt-centric checkout screen.
   - `Priority Score: 79`
   - rejected because it would widen scope well beyond the current Step 06 closure slice

3. Leave the pending-payment panel on compatibility checkout-session until a later full rewrite.
   - `Priority Score: 28`
   - rejected because it would keep the main billing interaction surface behind the old aggregate model even after formal detail APIs existed

### Chosen Action

Action 1 was selected because it moves the active pending-payment workbench toward the formal payment model immediately without destabilizing the current UI contract.

## Root Cause Summary

### 1. The Billing History Was Formalized, But The Pending-Payment Detail Panel Was Not

The previous slice had already moved billing history composition onto:

- formal order detail
- formal payment-method detail
- formal latest payment-attempt detail

But the interactive pending-payment panel still loaded only:

- `GET /portal/commerce/orders/{order_id}/checkout-session`

### 2. Compatibility Checkout Session Still Owned Reference And Rail Identity

That meant the visible pending-order workbench still preferred:

- compatibility reference ids
- compatibility provider rail labels
- compatibility method posture

even when canonical payment-attempt and payment-method data already existed.

### 3. The Repository Had No Dedicated Formal-First Checkout Detail Boundary

The billing repository exposed only `getBillingCheckoutSession(orderId)`, so the page had no clean place to consume a formal-first composition model.

## Implemented Fixes

- added `getBillingCheckoutDetail(orderId)` to the Portal billing repository
- the new repository composition now loads, in formal-first order:
  - `GET /portal/commerce/orders/{order_id}`
  - `GET /portal/commerce/orders/{order_id}/payment-methods`
  - `GET /portal/commerce/payment-attempts/{payment_attempt_id}` when present
  - `GET /portal/commerce/orders/{order_id}/checkout-session` as the compatibility interactive rail
- introduced `BillingCheckoutDetail` as the local billing-panel composition type
- updated the billing page to keep the existing `Checkout session` workbench but prefer:
  - canonical latest attempt reference
  - canonical selected payment-method label
  - canonical selected payment-method provider
  in the panel header facts
- preserved the compatibility checkout-session driven method list and callback rehearsal area for now, since those flows are still compatibility-backed in the current runtime

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/types/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/repository/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed before the repository change because `billingRepository.getBillingCheckoutDetail` did not exist

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal exec tsc --noEmit`

## Current Assessment

### Closed In This Slice

- the Portal billing pending-payment panel no longer depends on compatibility checkout-session as its only data source
- canonical attempt reference and canonical payment-method identity now reach the main billing interaction surface
- the billing repository now exposes a clean formal-first detail boundary for future retry/payment-choice migration

### Still Open

- the interactive checkout method list remains compatibility checkout-session based
- provider callback rehearsal and manual settlement actions remain compatibility-era flows behind the payment-simulation posture gate
- the wider attempt-backed checkout interaction model is still not finished

## Next Slice Recommendation

1. Move the pending-order method/action list toward canonical payment-method and payment-attempt posture instead of compatibility checkout-session methods.
2. Decide whether Portal will keep order-scoped checkout-session as a temporary operator/lab bridge only, or formalize an attempt-scoped interactive checkout detail resource.
3. Continue pruning compatibility-only payment actions from the user-facing Portal surface as the formal payment model becomes complete.
