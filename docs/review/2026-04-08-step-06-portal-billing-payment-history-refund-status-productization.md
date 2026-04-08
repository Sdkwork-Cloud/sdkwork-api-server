# 2026-04-08 Step 06 Portal Billing Payment History Refund Status Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by productizing the payment-history panel description that still exposed `refund closure` wording on the tenant-facing Portal billing workbench, without changing payment-history contracts, billing ownership boundaries, or runtime behavior.

Execution boundary:

- keep payment-history rows, refund evidence, and billing repository composition unchanged
- keep Portal billing runtime behavior unchanged
- do not introduce new backend billing routes, refund models, or finance transitions

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `payment-history-refund-status-productization`
- Previous mode: `refund-history-description-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the visible payment-history description so Portal billing explains this finance panel with `refund status` rather than `refund closure`.
   - `Priority Score: 187`
   - highest commercialization gain with a bounded display-only surface and no runtime risk

2. Leave the old `refund closure` wording visible because the underlying payment and refund evidence is already correct.
   - `Priority Score: 29`
   - rejected because the remaining term still reads like internal implementation language on a tenant-facing workbench

3. Broaden the slice into payment-history contracts or billing repository changes.
   - `Priority Score: 38`
   - rejected because the issue was wording only and widening the slice would add runtime churn without solving a functional defect

### Chosen Action

Action 1 was selected because it removes a visible commercialization leak immediately while preserving the already-verified payment-history data load, refund evidence rendering, and Portal finance runtime behavior.

## Root Cause Summary

### 1. The Payment-History Panel Still Exposed `Refund Closure` Terminology

The billing page still described payment history as keeping `refund closure` visible. That wording is weaker product language than `refund status` and reads closer to an internal finance workflow than a tenant-facing billing workbench.

### 2. The Runtime Facts Were Already Correct

The issue was not missing payment rows, broken refund evidence, or incorrect history rendering. The Portal billing workbench already loaded and displayed the relevant payment and refund facts correctly. The gap was strictly tenant-facing wording.

### 3. Shared I18n Coverage Needed To Match The Replacement

The new payment-history description needed to exist in both the billing page and the shared Portal i18n source contract. The `zh-CN` message catalog also needed a matching translation so the Portal surface would not fall back to English.

## Implemented Fixes

- replaced the payment-history panel description with the new product-facing `refund status` wording
- removed the retired `refund closure` phrase from the billing page source and shared Portal i18n contract
- updated shared Portal i18n and `zh-CN` translations so the new payment-history description is fully registered
- tightened the Portal billing-i18n, product, and commercial-api proof lanes so the new wording is required and the retired term is blocked

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
  - failed after the red-first test update because the shared Portal i18n source contract did not yet register the new payment-history description
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered `refund closure`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed after the red-first test update because the billing page source still contained `refund closure`

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- the Portal billing payment-history panel no longer exposes `refund closure` as tenant-facing copy
- shared Portal i18n and `zh-CN` now carry the new payment-history description required by the active billing workbench
- the focused proof lanes now guard this payment-history description contract against regression

### Still Open

- other low-level billing wording outside the current payment-history description cluster may still need future productization
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/é‹èˆµç€¯/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, or architecture ownership boundaries

## Next Slice Recommendation

1. Continue auditing whether tenant-facing billing surfaces still leak low-level finance review terminology.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized Portal billing finance baseline.
