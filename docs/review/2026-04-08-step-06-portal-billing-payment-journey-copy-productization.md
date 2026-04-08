# 2026-04-08 Step 06 Portal Billing Payment Journey Copy Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by cleaning up the remaining user-facing payment-journey wording in billing and recharge, without changing runtime contracts or action boundaries.

Execution boundary:

- keep the existing billing queue, checkout workbench, and recharge history surfaces intact
- keep formal order / payment-method / payment-attempt truth primary
- do not delete explicit bridge/manual actions from the runtime in this slice
- do not invent new backend payment flows or new frontend consoles

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `payment-journey-copy-productization`
- Previous mode: `queue-action-workbench-boundary-closure`
- Strategy switch: no

### Candidate Actions

1. Productize the remaining billing and recharge payment-journey copy so user guidance points to checkout workbench / checkout completion language instead of settlement/operator terminology.
   - `Priority Score: 168`
   - highest commercialization gain without widening scope or disturbing runtime boundaries

2. Remove the remaining simulation and operator-facing payment surfaces from Portal billing entirely.
   - `Priority Score: 91`
   - rejected because it widens the slice into structural product changes not yet backed by runtime removal

3. Leave the current wording in place and rely on the previous action-boundary fixes alone.
   - `Priority Score: 37`
   - rejected because the Portal would still describe tenant payment posture through settlement/operator language even after the workbench boundary had already been corrected

### Chosen Action

Action 1 was selected because Step 06 still had visible tenant-facing copy that lagged behind the actual product boundary. The Portal already routes the user through the checkout workbench, so the copy should describe that reality directly.

## Root Cause Summary

### 1. The User Journey Still Used Settlement Language After The Action Boundary Had Already Moved

After the previous slice:

- queue-row settlement ownership was removed
- explicit `settle_order` / `cancel_order` actions stayed inside the opened checkout workbench
- post-order guidance already pointed users to the checkout workbench

but several remaining user-visible copy paths still said:

- `Settle a subscription order`
- `Open billing queue`
- `workspace settles or cancels`

That left the Portal speaking in a pre-commercialization payment vocabulary even though the interaction boundary had already moved.

### 2. Billing Descriptions Still Framed Normal Tenant Flows Through Operator Terms

The billing page still described pending / failed / payment-history posture using phrases like:

- `provider callback review`
- `operator-facing audit timeline`

Those descriptions made the default tenant billing workbench sound more like an operator console than a productized customer payment surface.

### 3. The I18n Contract Needed To Move In Lockstep With The New Copy

Because the Portal i18n layer uses source-contract tests and shared registries, the copy productization work also required:

- new shared i18n key coverage
- new `zh-CN` coverage
- one regex relaxation so the proof lane matches the real multiline / trailing-comma `t(...)` call shape

## Implemented Fixes

- updated the Portal billing page to:
  - point checkout-session guidance to the checkout workbench and selected payment rail
  - point the selected-order empty state to the checkout workbench
  - replace the no-membership `Settle a subscription order` guidance with `Complete a subscription checkout`
  - productize pending / failed / payment-history descriptions away from settlement/operator jargon
- updated the Portal recharge page to:
  - replace the no-membership `Settle a subscription order` guidance with `Complete a subscription checkout`
  - rename the navigation CTA from `Open billing queue` to `Open billing workbench`
- updated shared Portal i18n registries and `zh-CN` translations for the new payment-journey copy
- updated source-contract tests for:
  - the new checkout-workbench guidance
  - the new membership activation copy
  - the new recharge CTA
  - the new pending / failed / payment-history copy
- relaxed the remaining billing-i18n regex to allow the actual multiline / trailing-comma `setCheckoutSessionStatus(t(...),)` shape

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
- `apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed after the red-first test update because billing still used the previous queue-session guidance and membership wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because billing still used the previous pending / failed / payment-history descriptions
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`
  - failed after the red-first test update because recharge still exposed `Open billing queue`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed once more after the production copy fix because the regex was still too strict for the real multiline / trailing-comma formatter output

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## Current Assessment

### Closed In This Slice

- billing and recharge now guide users through checkout-workbench / checkout-completion language instead of settlement phrasing
- the default billing lane descriptions and payment-history description now read as tenant product copy rather than operator-console copy
- shared Portal i18n and `zh-CN` now cover the new wording
- the billing i18n proof now matches the actual formatter output instead of creating a false negative

### Still Open

- explicit bridge/manual actions still remain in the billing runtime
- provider callback rehearsal still remains in the billing runtime
- compatibility checkout-session still remains as a bridge / fallback source
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/é‹èˆµç€¯/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal product copy, user guidance, and i18n/test coverage; it did not change route authority, runtime ownership, backend API truth, or architecture contracts

## Next Slice Recommendation

1. Continue auditing user-facing billing copy for any remaining operator-specific concepts that are no longer the default tenant journey.
2. Evaluate whether the explicit simulation surfaces should be relabeled or moved into a clearer operator-only boundary once the product surface is otherwise clean.
3. Continue Step 06 commercialization closure until release evidence shows the Portal billing journey is productized from first CTA to final timeline copy.
