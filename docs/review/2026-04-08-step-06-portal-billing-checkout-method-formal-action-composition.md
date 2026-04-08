# 2026-04-08 Step 06 Portal Billing Checkout Method Formal Action Composition Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by closing the next remaining payment-display gap inside the billing checkout workbench.

Execution boundary:

- keep the current `Checkout session` panel and compatibility payload in place
- stop letting compatibility `checkout_session.methods` define the only method/action identity visible to the user
- normalize canonical `payment_methods` plus latest `payment_attempt` posture into the method-card and callback-rail surface

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `checkout-method-formal-action-closure`
- Previous mode: `pending-payment-detail-closure`
- Strategy switch: no

### Candidate Actions

1. Add a billing-layer checkout-method normalization helper and move the current page onto that formal-first view model.
   - `Priority Score: 141`
   - highest architecture value for the smallest safe write surface

2. Replace the whole Portal checkout workbench with a brand-new attempt-scoped interaction screen.
   - `Priority Score: 82`
   - rejected because it would widen scope beyond the current closure slice

3. Keep showing compatibility checkout-session methods until the backend publishes a fully attempt-scoped interactive contract.
   - `Priority Score: 31`
   - rejected because it would leave the most user-visible payment action surface coupled to the old bridge model

### Chosen Action

Action 1 was selected because it closes the highest-value frontend coupling immediately while keeping the current runtime contract stable.

## Root Cause Summary

### 1. Formal Detail Existed, But Method Cards Still Came From The Compatibility Bridge

The previous slice already loaded:

- canonical order detail
- canonical payment methods
- canonical latest payment attempt

But the billing page still rendered:

- checkout method cards from `checkoutSession.methods`
- callback rehearsal rail choices from `checkoutSession.methods`

### 2. Compatibility Method Identity Could Drift From The Real Payment Method Model

That meant the visible Portal payment surface could still anchor on:

- compatibility-only method ids
- compatibility-only provider rail posture
- compatibility-only callback readiness

even when canonical payment methods and canonical attempt references already said something more accurate.

### 3. There Was No Billing-Level Normalization Boundary

The repository exposed formal inputs and compatibility session side by side, but no local billing helper translated them into a single formal-first checkout method list for the page to consume.

## Implemented Fixes

- added `buildBillingCheckoutMethods(...)` in the Portal billing services layer
- introduced `checkout_methods` on `BillingCheckoutDetail`
- updated repository composition so `getBillingCheckoutDetail(orderId)` now returns:
  - canonical order
  - canonical payment methods
  - canonical latest payment attempt
  - compatibility checkout session
  - formal-first normalized checkout methods
- the normalization logic now:
  - keeps compatibility operator actions as a bridge
  - projects canonical payment methods into checkout methods for the open pending-payment state
  - prefers canonical selected method identity and latest-attempt reference for provider rails
  - falls back to compatibility session methods only when canonical provider rails are missing
- updated the billing page to:
  - render method cards from `checkoutDetail.checkout_methods`
  - choose provider callback rehearsal rails from `checkoutDetail.checkout_methods`
  - keep compatibility `checkoutSession.methods` only as a fallback when formal detail is absent

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/types/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/services/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/repository/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed before the implementation because the page still filtered callback rails from `checkoutSession.methods` and the repository did not return normalized `checkout_methods`

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal exec tsc --noEmit`

## Current Assessment

### Closed In This Slice

- the Portal billing checkout method/action surface is no longer compatibility-only
- canonical payment-method identity now reaches both the visible method cards and the callback rehearsal rail chooser
- the billing module now has a dedicated normalization boundary for future attempt-backed checkout evolution

### Still Open

- the underlying interactive settlement/callback mutation path still uses the compatibility-era order-scoped bridge
- the Portal billing workbench is still not attempt-scoped end to end
- retry and payment-choice journeys still need broader formal-model closure

## Next Slice Recommendation

1. Move the retry/payment-choice lane from order-scoped compatibility flows toward explicit payment-attempt creation and provider checkout launch.
2. Decide whether the Portal workbench should keep compatibility operator actions as a temporary lab bridge only.
3. Continue shrinking user-visible compatibility behaviors until the pending-payment panel is fully attempt-backed.
