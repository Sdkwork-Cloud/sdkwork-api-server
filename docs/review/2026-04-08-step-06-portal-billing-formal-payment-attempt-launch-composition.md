# 2026-04-08 Step 06 Portal Billing Formal Payment Attempt Launch Composition Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by moving the next user-visible payment action off the compatibility bridge and onto the formal commerce contract.

Execution boundary:

- keep the current Portal `Checkout session` workbench and compatibility settlement/callback bridge in place
- stop making provider checkout launch depend on compatibility-only `checkout_session` interaction posture
- expose the real formal payment-attempt creation route through the frontend SDK, repository, and billing page

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `formal-payment-attempt-launch-closure`
- Previous mode: `checkout-method-formal-action-closure`
- Strategy switch: no

### Candidate Actions

1. Wire the existing formal backend payment-attempt creation contract into the Portal billing checkout workbench.
   - `Priority Score: 147`
   - highest architecture value with the smallest stable write surface

2. Build a brand-new attempt-centered checkout workbench before exposing launch actions.
   - `Priority Score: 79`
   - rejected because it would widen scope beyond the current closure slice

3. Keep provider launch entirely on the compatibility checkout-session bridge until all providers are formalized.
   - `Priority Score: 34`
   - rejected because the canonical Stripe path already exists and should be used now

### Chosen Action

Action 1 was selected because it closes a real commercial gap immediately without widening the Portal UI into a larger redesign.

## Root Cause Summary

### 1. Formal Payment-Attempt Launch Already Existed On The Backend

The runtime already exposed:

- `GET /portal/commerce/orders/{order_id}/payment-attempts`
- `POST /portal/commerce/orders/{order_id}/payment-attempts`

But the Portal TypeScript client and billing repository did not surface those routes.

### 2. The Billing Workbench Could Show Canonical Methods Without Using The Canonical Launch Path

The previous slices already moved the billing workbench toward:

- canonical order detail
- canonical payment methods
- canonical latest payment attempt

Yet the actual provider-launch action still had no formal frontend path, which meant the user-visible workbench could still stall at the bridge boundary.

### 3. Shared Portal I18n Did Not Yet Cover The New Provider-Launch Copy

Once the billing page gained formal provider-launch status and button copy, the shared Portal message registry and `zh-CN` catalog also needed to be updated so the new flow did not regress localization posture.

## Implemented Fixes

- added `PortalCommercePaymentAttemptCreateRequest` to the shared Portal TypeScript types package
- added Portal API client methods:
  - `listPortalCommercePaymentAttempts(orderId)`
  - `createPortalCommercePaymentAttempt(orderId, input)`
- added billing repository support:
  - `createBillingPaymentAttempt(orderId, input)`
- updated the Portal billing page so supported provider rails now:
  - reopen the latest canonical payment attempt when it already has `checkout_url`
  - create a new formal payment attempt when the current rail is eligible and no active checkout URL exists
  - pass canonical `success_url` and `cancel_url` derived from the current Portal route
- explicitly limited the formal launch path to Stripe hosted-checkout rails:
  - `action === 'provider_handoff'`
  - `availability === 'available'`
  - `provider === 'stripe'`
  - `channel === 'hosted_checkout'`
- added shared Portal i18n keys and `zh-CN` translations for:
  - provider-launch status messages
  - provider-launch button states

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/repository/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
- `apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed before the implementation because formal payment-attempt launch was not exposed through the Portal TypeScript client / billing repository / billing page path
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed before the i18n completion because the new provider-launch strings were not yet present in the shared Portal message catalog

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal exec tsc --noEmit`

## Current Assessment

### Closed In This Slice

- the Portal billing workbench can now use the formal payment-attempt create contract to launch provider checkout
- canonical latest payment-attempt detail now drives both reopen and fresh-launch behavior for the supported formal rail
- the new payment-attempt launch posture is covered by shared Portal i18n instead of raw page-local English fallback

### Still Open

- non-Stripe provider handoff rails still remain on the bridge posture because the backend formal attempt-creation lane is not yet wired for them
- operator settlement and provider callback simulation are still intentionally compatibility-era bridge actions
- the pending-payment workbench is still not fully attempt-scoped end to end

## Next Slice Recommendation

1. Expose attempt history and retry posture more directly inside the billing workbench so users can understand whether they are reopening an existing checkout or creating a fresh attempt.
2. Continue shrinking order-scoped compatibility actions until only explicit operator-only bridge behavior remains.
3. Expand the formal provider-launch lane only when the backend payment-attempt creation path is canonical for additional providers, not before.
