# 2026-04-08 Step 06 Portal Billing Payment Attempt History Composition Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by making the billing checkout workbench show the real order-scoped payment-attempt history instead of only the single `latest_payment_attempt` view.

Execution boundary:

- keep the existing Portal `Checkout session` workbench layout intact
- keep compatibility checkout-session detail as the current operator bridge surface
- expose canonical formal payment-attempt history inside the same workbench without widening the UI into a new payment console

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `formal-payment-attempt-history-closure`
- Previous mode: `formal-payment-attempt-launch-closure`
- Strategy switch: no

### Candidate Actions

1. Compose canonical order-scoped payment-attempt history into the existing billing checkout panel.
   - `Priority Score: 153`
   - closes a real user-visible commercial gap with the smallest stable write surface

2. Replace the current checkout workbench with an attempt-first standalone payment console.
   - `Priority Score: 74`
   - rejected because it widens Step 06 scope and delays closure

3. Keep showing only `latest_payment_attempt` until the entire checkout bridge is removed.
   - `Priority Score: 31`
   - rejected because it hides retry posture and attempt chronology even though the canonical list route already exists

### Chosen Action

Action 1 was selected because the backend already exposes the order-scoped canonical list route, and the Portal should surface that truth immediately instead of forcing operators and finance users to infer retry history from a single latest attempt.

## Root Cause Summary

### 1. The Billing Workbench Still Collapsed Canonical Attempt History Into One Record

The repository had already begun composing canonical order detail and canonical payment methods, but the page still only consumed:

- `checkoutDetail.latest_payment_attempt`
- compatibility `checkout_session`

That left no visible place to understand:

- how many payment attempts already happened
- whether the current order is reopening an existing provider checkout or retrying after a prior failure
- which attempt is the newest canonical attempt

### 2. One Repository Path Still Assumed A Single Attempt Shape

After switching `getBillingCheckoutDetail(...)` to the formal list route, one existing regression fixture still returned an object payload instead of an array payload. That exposed a real robustness gap:

- `sortBillingPaymentAttempts(...)` trusted runtime shape too strongly
- a non-array payload would throw before the workbench could render

### 3. Shared Portal I18n Did Not Yet Cover Attempt-History Copy

The new payment-attempt history surface required shared message keys and `zh-CN` translations for:

- `Payment attempts`
- `Latest attempt`
- attempt sequence / status / timing copy

Without that, the new surface would regress the shared Portal localization posture.

## Implemented Fixes

- hardened `sortBillingPaymentAttempts(...)` so a malformed runtime payload now degrades to an empty attempt list instead of throwing
- aligned the Portal source-contract regression fixture for pending-order checkout detail with the formal route:
  - `GET /portal/commerce/orders/{order_id}/payment-attempts`
- updated the Portal billing page to:
  - read `checkoutDetail.payment_attempts`
  - render a dedicated `Payment attempts` section inside the existing `Checkout session` panel
  - mark the canonical newest record with `Latest attempt`
  - show per-attempt status, sequence, provider reference, initiated time, updated time, and provider/channel posture
  - show the error message inline when an attempt failed with provider-returned detail
- added shared Portal i18n keys and `zh-CN` translations for the new attempt-history labels and empty-state copy

## Files Touched In This Slice

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
  - failed before the fix because:
    - the page source still did not reference `checkoutDetail?.payment_attempts`
    - the pending-order fixture still encoded the old single-attempt route shape
    - `sortBillingPaymentAttempts(...)` threw `paymentAttempts.slice is not a function` when it received a non-array payload
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed before the fix because the shared Portal message registry still lacked `Payment attempts` and `Latest attempt`

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal exec tsc --noEmit`

## Current Assessment

### Closed In This Slice

- the Portal checkout workbench now shows canonical attempt chronology instead of hiding retry posture behind a single latest-attempt record
- the source-contract suite is aligned with the formal order-scoped attempt-list contract
- the attempt-history surface is localized through the shared Portal i18n boundary instead of page-local raw strings
- the repository path is safer against malformed runtime attempt payloads

### Still Open

- the billing workbench still depends on compatibility checkout-session structure for the larger panel shell and operator bridge actions
- attempt history is currently read-only; there is still no explicit attempt-level action strip beyond the existing launch / reopen behavior
- non-Stripe provider rails remain intentionally outside the formal attempt-launch lane

## Next Slice Recommendation

1. Reduce remaining compatibility dependence in the checkout workbench so canonical attempt and canonical checkout-method posture drive more of the panel state directly.
2. Add clearer retry / reopen decision copy that explains when the workbench will reuse the latest canonical `checkout_url` versus create a fresh attempt.
3. Expand attempt-level operational detail only when the backend formal contract exposes more provider-safe action semantics, not by inventing frontend-only state.
