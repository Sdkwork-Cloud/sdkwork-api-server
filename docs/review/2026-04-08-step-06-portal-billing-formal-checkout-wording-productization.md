# 2026-04-08 Step 06 Portal Billing Formal Checkout Wording Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by productizing the active tenant-facing `formal checkout` wording in the Portal billing workbench, without changing billing contracts, checkout launch logic, or runtime behavior.

Execution boundary:

- keep the real order, payment-method, checkout-session, and payment-attempt flows unchanged
- keep Portal billing runtime behavior unchanged
- do not introduce new backend billing routes, SDK methods, or commerce payload changes

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `formal-checkout-wording-productization`
- Previous mode: `fallback-reason-description-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the visible `formal checkout` wording in the active billing workbench description, fallback guidance, and provider launch status.
   - `Priority Score: 185`
   - highest commercialization gain with a bounded display-only surface and no runtime risk

2. Leave the old `formal` wording visible because the runtime already uses the canonical checkout path correctly.
   - `Priority Score: 24`
   - rejected because the tenant-facing Portal surface still leaked internal implementation vocabulary even though the runtime facts were already correct

3. Broaden the slice into formal payment-attempt, checkout-session, or backend contract refactors.
   - `Priority Score: 31`
   - rejected because the defect was wording only and widening the slice would add avoidable churn without solving a functional issue

### Chosen Action

Action 1 was selected because it removes a visible commercialization leak immediately while preserving the already-verified Portal billing runtime behavior and checkout launch flow.

## Root Cause Summary

### 1. Active Tenant-Facing Checkout Copy Still Exposed Internal `Formal` Vocabulary

The billing workbench still rendered `Formal checkout keeps ...`, `No formal guidance is available ...`, and provider launch status messages that used `formal {provider} checkout ...`. Those phrases center internal implementation naming rather than the tenant-facing product surface.

### 2. The Runtime Checkout Facts Were Already Correct

The issue was not missing payment attempts, broken checkout launch logic, or incorrect billing state. The Portal billing workbench already loaded and displayed the relevant checkout facts correctly. The gap was strictly tenant-facing wording.

### 3. Shared I18n Coverage Needed To Stay In Lockstep

The new wording had to exist in both the billing page and the shared Portal i18n source contract. The `zh-CN` message catalog also needed matching translations so the Portal surface would not fall back to English.

## Implemented Fixes

- replaced the payment-method panel description with product-facing `Checkout workbench` wording
- replaced the fallback guidance with `No checkout guidance is available for this order yet.`
- replaced the provider launch status and missing-link message with checkout-focused wording and `checkout link`
- removed the retired `formal` phrases from the billing page source and shared Portal i18n contract
- updated shared Portal i18n and `zh-CN` translations so the new checkout wording is fully registered
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
  - failed after the red-first test update because the billing page and shared Portal i18n contract still contained the older `formal` checkout wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered the older `Formal checkout keeps ...` description
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed after the red-first test update because the billing page source still contained the older `formal {provider} checkout` launch wording

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- the Portal billing workbench no longer exposes active tenant-facing `formal checkout` wording in the payment-method description, fallback guidance, or provider-launch status messaging
- shared Portal i18n and `zh-CN` now carry the new checkout wording required by the active billing workbench
- the focused proof lanes now guard this checkout wording contract against regression

### Still Open

- other billing wording outside the current `formal checkout` cluster may still need future productization
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, or architecture ownership boundaries

## Next Slice Recommendation

1. Continue auditing whether tenant-facing billing surfaces still leak low-level checkout or finance implementation terminology.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized Portal billing baseline.
