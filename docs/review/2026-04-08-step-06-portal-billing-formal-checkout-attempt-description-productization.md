# 2026-04-08 Step 06 Portal Billing Formal Checkout Attempt Description Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by productizing the active tenant-facing checkout-attempt history description that still exposed `Formal order-scoped` implementation vocabulary in the Portal billing workbench, without changing billing contracts, payment-attempt composition, or runtime behavior.

Execution boundary:

- keep the real order-scoped payment-attempt history flow unchanged
- keep Portal billing runtime behavior unchanged
- do not introduce new backend billing routes, SDK methods, or commerce payload changes

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `formal-checkout-attempt-description-productization`
- Previous mode: `formal-checkout-wording-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the visible checkout-attempt history description so the tenant-facing billing workbench stops saying `Formal order-scoped checkout attempts ...`.
   - `Priority Score: 176`
   - highest commercialization gain with a bounded display-only surface and no runtime risk

2. Leave the old `Formal order-scoped` wording visible because the payment-attempt history data is already correct.
   - `Priority Score: 22`
   - rejected because the tenant-facing Portal surface still leaked internal implementation naming even though the runtime facts were already correct

3. Broaden the slice into payment-attempt runtime, repository, or backend contract refactors.
   - `Priority Score: 29`
   - rejected because the defect was wording only and widening the slice would add avoidable churn without solving a functional issue

### Chosen Action

Action 1 was selected because it removes a visible commercialization leak immediately while preserving the already-verified Portal billing payment-attempt history behavior.

## Root Cause Summary

### 1. The Checkout-Attempt History Panel Still Exposed `Formal Order-Scoped` Vocabulary

The billing page still described the active checkout-attempt panel as `Formal order-scoped checkout attempts keep ...`. That phrase centers internal implementation naming rather than the tenant-facing product surface.

### 2. The Runtime Payment-Attempt Facts Were Already Correct

The issue was not missing payment-attempt history, broken latest-attempt ordering, or incorrect billing state. The Portal billing workbench already loaded and displayed the relevant payment-attempt facts correctly. The gap was strictly tenant-facing wording.

### 3. Shared I18n Coverage Needed To Stay In Lockstep

The new checkout-attempt description had to exist in both the billing page and the shared Portal i18n source contract. The `zh-CN` message catalog also needed a matching translation so the Portal surface would not fall back to English.

## Implemented Fixes

- replaced the checkout-attempt panel description with `Checkout attempts keep checkout access, retries, and checkout references visible inside the same workbench.`
- removed the retired `Formal order-scoped` wording from the billing page source and shared Portal i18n contract
- updated shared Portal i18n and `zh-CN` translations so the new checkout-attempt description is fully registered
- tightened the Portal billing-i18n, product, and commercial-api proof lanes so the new wording is required and the retired wording is blocked

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
- `apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed after the red-first test update because the shared Portal i18n source contract did not yet register the new checkout-attempt description
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered the older `Formal order-scoped checkout attempts ...` wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed after the red-first test update because the billing page source still contained the older `Formal order-scoped checkout attempts ...` description

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- the Portal billing checkout-attempt history panel no longer exposes `Formal order-scoped` wording on the active tenant-facing surface
- shared Portal i18n and `zh-CN` now carry the new checkout-attempt description required by the active billing workbench
- the focused proof lanes now guard this checkout-attempt description contract against regression

### Still Open

- other billing wording outside the current checkout-attempt description cluster may still need future productization
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, or architecture ownership boundaries

## Next Slice Recommendation

1. Continue auditing whether tenant-facing billing surfaces still leak low-level checkout or finance implementation terminology.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized Portal billing baseline.
