# 2026-04-08 Step 06 Portal Billing Payment Attempt Vocabulary Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by productizing the checkout-history wording that still exposed `payment attempt` terminology on the tenant-facing Portal billing workbench, without changing payment-attempt contracts, backend ownership boundaries, or runtime behavior.

Execution boundary:

- keep `payment_attempt_id`, `payment_attempts` payloads, and formal payment-attempt launch flows unchanged
- keep existing first-attempt, retry, and resume launch branches unchanged
- keep Portal billing runtime behavior unchanged
- do not introduce new backend billing routes, payment state transitions, or checkout models

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `payment-attempt-vocabulary-productization`
- Previous mode: `provider-checkout-action-vocabulary-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the visible checkout-history wording so Portal billing renders `Checkout attempts` and related checkout-attempt guidance instead of `Payment attempts`.
   - `Priority Score: 188`
   - highest commercialization gain with a bounded display-only surface and no runtime risk

2. Leave `payment attempt` visible because the canonical backend contracts still use payment-attempt naming.
   - `Priority Score: 36`
   - rejected because Step 06 still requires the tenant-facing billing workbench to speak in product checkout language rather than internal commerce contract terminology

3. Rename payment-attempt contracts across the stack.
   - `Priority Score: 43`
   - rejected because it would widen the slice into contract churn without solving a runtime bug

### Chosen Action

Action 1 was selected because it removes a visible commercialization leak immediately while preserving the already-verified payment-attempt payloads, launch decisions, and Portal billing runtime behavior.

## Root Cause Summary

### 1. The Checkout History Workbench Still Exposed Payment-Attempt Terminology

After the earlier billing vocabulary slices, the checkout workbench still rendered `Payment attempts`, `No payment attempts recorded yet`, and payment-attempt phrasing in retry guidance. That wording described internal commerce implementation instead of the tenant-facing checkout history the user is actually reviewing.

### 2. The Runtime Facts Were Already Correct

The issue was not missing attempt records, broken retry logic, or incorrect provider launch routing. The billing page already loaded canonical attempt history and used the same formal payment-attempt path for first-attempt and retry launches. The gap was strictly tenant-facing wording.

### 3. Shared I18n Coverage Needed To Match The Replacement

The new checkout-attempt wording needed to exist in both the billing page and the shared Portal i18n source contract. The `zh-CN` message catalog also needed matching entries so the Portal surface would not fall back to English.

## Implemented Fixes

- replaced `Payment attempts` with `Checkout attempts` on the checkout-history panel
- replaced empty-state and explanatory copy so the workbench now speaks in checkout-attempt language
- replaced retry and first-attempt guidance so the workbench now says `fresh checkout attempt` and `No {provider} checkout attempt exists yet`
- replaced the failed-payment summary detail so it now describes checkout attempts that closed on the failure path
- updated shared Portal i18n and `zh-CN` translations so the new checkout-attempt vocabulary is fully registered
- tightened the Portal billing-i18n, product, and commercial-api proof lanes so the new wording is required and the retired payment-attempt wording is blocked

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
  - failed after the red-first test update because the shared Portal i18n source contract still registered the old payment-attempt wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered the old payment-attempt wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed after the red-first test update because the billing page source still contained `Payment attempts`

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- the Portal billing checkout-history workbench no longer exposes `payment attempt` wording as tenant-facing copy
- shared Portal i18n and `zh-CN` now carry the new checkout-attempt wording required by the active checkout workbench
- the focused proof lanes now guard this checkout-attempt vocabulary contract against regression

### Still Open

- other low-level billing wording outside the current checkout-attempt cluster may still need future productization
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, or architecture ownership boundaries

## Next Slice Recommendation

1. Continue auditing whether tenant-facing billing surfaces still leak provider-confirmation or refund-history implementation terms.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized Portal checkout-history baseline.
