# 2026-04-08 Step 06 Portal Billing Operator Bridge Visibility Closure Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by reducing the remaining operator-bridge visibility inside the pending-payment checkout workbench.

Execution boundary:

- keep the existing Portal `Checkout session` workbench intact
- keep formal order/payment-method/payment-attempt truth primary
- keep queue settlement and provider callback rehearsal available only under the existing payment-simulation posture
- do not invent a new backend payment console or new attempt-scoped contracts

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `operator-bridge-visibility-closure`
- Previous mode: `formal-checkout-presentation-shell`
- Strategy switch: no

### Candidate Actions

1. Remove operator-bridge posture from the default checkout method list and rail summary while preserving the existing simulation gate.
   - `Priority Score: 174`
   - highest commercialization gain without widening scope or inventing backend semantics

2. Rebuild the pending-payment workbench into a dedicated operator-versus-customer split console.
   - `Priority Score: 83`
   - rejected because it widens Step 06 scope and introduces a larger UI rewrite not yet required by the runtime

3. Leave the current mixed presentation in place and rely on `payment_simulation_enabled` alone.
   - `Priority Score: 34`
   - rejected because the page still made operator-bridge posture look like canonical payment truth even after formal-first shell composition already existed

### Chosen Action

Action 1 was selected because the current Portal runtime already has enough information to stop presenting operator bridge posture as the default payment rail summary, while still preserving the intentional lab-only controls behind the existing simulation gate.

## Root Cause Summary

### 1. The Checkout Method Grid Still Mixed Operator Bridge Posture Into The Normal User Payment Surface

Even after the earlier Step 06 slices introduced:

- canonical order detail
- canonical checkout-method composition
- canonical payment-attempt launch and history
- canonical presentation of reference, status, and guidance

the main workbench method grid still rendered `settle_order` inside the same default checkout method list as the formal provider handoff rails.

### 2. The Payment Rail Summary Still Hardcoded Compatibility Bridge Rows

The separate `Payment rail` workspace panel still rendered:

- `Local desktop mode`
- `Operator settlement`
- `Server mode handoff`

That made the compatibility bridge look like first-class customer payment truth instead of an operator/lab posture.

### 3. Shared Portal I18n Did Not Yet Cover The Productized Replacement Copy

Once the page stopped using `Operator settlement` as the default user-facing label, the shared Portal i18n registry and `zh-CN` catalog also needed the replacement `Manual settlement` label plus the new formal rail summary description.

## Implemented Fixes

- updated the Portal billing page to:
  - filter visible checkout methods so `settle_order` does not remain in the default method grid when simulation mode is off
  - keep the existing callback rehearsal surface gated by `payment_simulation_enabled`
  - keep the queue-level settle action in the intentional simulation lane
  - rename the remaining operator-settlement page copy to `Manual settlement`
  - stop the `Payment rail` panel from hardcoding operator bridge rows
  - make the `Payment rail` panel summarize only formal-first rail facts:
    - primary rail
    - current selected reference
    - payable price
- adjusted compatibility fallback in the page so `manual_lab` provider posture no longer becomes the default rail summary when simulation mode is not active
- updated shared Portal i18n registries and `zh-CN` translations for:
  - `Manual settlement`
  - the new formal rail summary description

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
- `apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed before the fix because the billing page still mapped `checkoutMethods` directly and had no explicit visibility filter for operator bridge settlement posture
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed before the fix because the billing page still contained `Operator settlement` as user-facing page copy
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed before the fix because shared Portal i18n did not yet include `Manual settlement`

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## Current Assessment

### Closed In This Slice

- the default user-facing checkout method grid no longer treats operator settlement as part of the normal formal payment rail presentation
- the `Payment rail` summary panel now stays aligned with formal checkout posture instead of mixing in operator bridge rows
- the billing page no longer uses `Operator settlement` as the default customer-facing payment label
- shared Portal i18n now covers the updated copy

### Still Open

- queue-level `Settle order` remains available in the intentional simulation posture
- provider callback rehearsal remains available in the intentional simulation posture
- compatibility checkout-session is still retained as a bridge and fallback source
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal page visibility, copy, and default presentation posture; it did not change route authority, runtime ownership, backend API truth, or architecture boundary contracts

## Next Slice Recommendation

1. Continue removing compatibility bridge ownership from the main checkout workbench only where the backend already exposes canonical truth.
2. Decide whether queue-level settlement and callback rehearsal should move behind a clearer operator-only boundary instead of remaining inside the billing user workbench.
3. Continue Step 06 commercialization closure until release evidence can show that the pending-payment workbench is formal-first end to end.
