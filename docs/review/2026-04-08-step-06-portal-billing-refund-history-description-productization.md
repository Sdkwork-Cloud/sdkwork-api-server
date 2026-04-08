# 2026-04-08 Step 06 Portal Billing Refund History Description Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by productizing the refund-history panel description that still exposed closed-loop/operator-style wording on the tenant-facing Portal billing workbench, without changing refund contracts, billing ownership boundaries, or runtime behavior.

Execution boundary:

- keep refund-history rows, payment-method evidence, and resulting order-status rendering behavior unchanged
- keep billing repository composition unchanged
- keep Portal billing runtime behavior unchanged
- do not introduce new backend billing routes, refund models, or order-state transitions

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `refund-history-description-productization`
- Previous mode: `payment-attempt-vocabulary-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the visible refund-history description so Portal billing explains the panel as finance evidence that keeps completed refund outcomes, payment method evidence, and resulting order status visible.
   - `Priority Score: 192`
   - highest commercialization gain with a bounded display-only surface and no runtime risk

2. Leave the old wording visible because refund review is still based on canonical billing evidence.
   - `Priority Score: 32`
   - rejected because the remaining `closed-loop` and provider-verification phrasing still leaks internal review language into the tenant-facing Portal surface

3. Broaden the slice into refund contract or repository changes.
   - `Priority Score: 41`
   - rejected because the issue was copy only and widening the slice would add runtime churn without solving a functional defect

### Chosen Action

Action 1 was selected because it removes a visible commercialization leak immediately while preserving the already-verified refund-history data load, billing evidence rendering, and Portal finance runtime behavior.

## Root Cause Summary

### 1. The Refund-History Panel Still Exposed Closed-Loop Review Wording

The billing page still described refund history as something that `isolates closed-loop refund outcomes` so operators could verify provider, checkout reference, and final order state. That wording described internal finance review mechanics rather than the product surface the tenant is using.

### 2. The Runtime Facts Were Already Correct

The issue was not missing refund rows, broken payment-method evidence, or incorrect order-status rendering. The Portal billing workbench already loaded and displayed the relevant refund evidence correctly. The gap was strictly tenant-facing wording.

### 3. Shared I18n Coverage Needed To Match The New Description

The new refund-history description needed to exist in both the billing page and the shared Portal i18n source contract. The `zh-CN` message catalog also needed a matching translation so the Portal surface would not fall back to English.

## Implemented Fixes

- replaced the refund-history panel description with the new product-facing explanation centered on completed refund outcomes, payment method evidence, and resulting order status
- removed the retired `closed-loop refund outcomes` and provider-verification wording from the billing page source
- updated shared Portal i18n and `zh-CN` translations so the new refund-history description is fully registered
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
  - failed after the red-first test update because the shared Portal i18n source contract did not yet register the new refund-history description
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered the older closed-loop refund wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed after the red-first test update because the billing page source still contained `closed-loop refund outcomes`

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- the Portal billing refund-history panel no longer exposes closed-loop/operator review wording as tenant-facing copy
- shared Portal i18n and `zh-CN` now carry the new refund-history description required by the active billing workbench
- the focused proof lanes now guard this refund-history description contract against regression

### Still Open

- other low-level billing wording outside the current refund-history description cluster may still need future productization
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/é‹èˆµç€¯/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, or architecture ownership boundaries

## Next Slice Recommendation

1. Continue auditing whether tenant-facing billing surfaces still leak low-level refund or finance review terminology.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized Portal billing finance baseline.
