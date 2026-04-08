# 2026-04-08 Step 06 Portal Billing Failed Payment Description Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by productizing the active tenant-facing failed-payment lane description that still exposed `isolates` operational vocabulary in the Portal billing workbench, without changing billing contracts, failed-lane composition, or runtime behavior.

Execution boundary:

- keep the real failed-payment lane classification and checkout-attempt follow-up flow unchanged
- keep Portal billing runtime behavior unchanged
- do not introduce new backend billing routes, SDK methods, or commerce payload changes

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `failed-payment-description-productization`
- Previous mode: `commercial-account-description-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the visible failed-payment lane description so the tenant-facing Portal billing workbench stops saying `Failed payment isolates ...`.
   - `Priority Score: 183`
   - highest commercialization gain with a bounded display-only surface and no runtime risk

2. Leave the old `isolates` wording visible because the failed-payment lane behavior is already correct.
   - `Priority Score: 24`
   - rejected because the tenant-facing Portal surface still leaked internal operational wording even though the runtime facts were already correct

3. Broaden the slice into failed-payment runtime, repository, or backend contract refactors.
   - `Priority Score: 30`
   - rejected because the defect was wording only and widening the slice would add avoidable churn without solving a functional issue

### Chosen Action

Action 1 was selected because it removes a visible commercialization leak immediately while preserving the already-verified Portal billing failed-payment behavior.

## Root Cause Summary

### 1. The Failed-Payment Lane Still Exposed `Isolates` Operational Vocabulary

The billing page still described the active failed-payment lane as `Failed payment isolates checkout attempts that need coupon updates, a different payment method, or a fresh checkout attempt.` That phrase centers internal operational handling rather than the tenant-facing product surface.

### 2. The Runtime Failed-Payment Facts Were Already Correct

The issue was not missing failed orders, broken retry guidance, or incorrect order-lane classification. The Portal billing workbench already loaded and displayed the relevant failed-payment facts correctly. The gap was strictly tenant-facing wording.

### 3. Shared I18n Coverage Needed To Stay In Lockstep

The new failed-payment description had to exist in both the billing page and the shared Portal i18n source contract. The `zh-CN` message catalog also needed a matching translation so the Portal surface would not fall back to English.

## Implemented Fixes

- replaced the failed-payment lane description with `Failed payment keeps checkout attempts that need coupon updates, a different payment method, or a fresh checkout visible for follow-up.`
- removed the retired `Failed payment isolates ...` wording from the billing page source and shared Portal i18n contract
- updated shared Portal i18n and `zh-CN` translations so the new failed-payment description is fully registered
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
  - failed after the red-first test update because the shared Portal i18n source contract did not yet register the new failed-payment description
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered the older `Failed payment isolates ...` wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed after the red-first test update because the billing page source still contained the older `Failed payment isolates ...` description

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `223 / 223` passing

## Current Assessment

### Closed In This Slice

- the Portal billing failed-payment lane no longer exposes `isolates` wording on the active tenant-facing surface
- shared Portal i18n and `zh-CN` now carry the new failed-payment description required by the active billing workbench
- the focused proof lanes now guard this failed-payment description contract against regression

### Still Open

- other billing wording outside the current failed-payment description cluster may still need future productization
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, or architecture ownership boundaries

## Next Slice Recommendation

1. Continue auditing whether tenant-facing billing surfaces still leak low-level finance or operational terminology.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized Portal billing baseline.
