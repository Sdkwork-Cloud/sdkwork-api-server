# 2026-04-08 Step 06 Portal Billing Provider Checkout Action Vocabulary Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by productizing the checkout-action wording that still exposed `provider checkout` terminology on the tenant-facing Portal billing workbench, without changing checkout launch contracts, backend ownership boundaries, or runtime behavior.

Execution boundary:

- keep `provider_handoff`, payment-attempt launch decisions, and checkout URL sourcing unchanged
- keep existing first-attempt, retry, and resume launch branches unchanged
- keep Portal billing runtime behavior unchanged
- do not introduce new backend billing routes, launch states, or checkout models

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `provider-checkout-action-vocabulary-productization`
- Previous mode: `payment-update-reference-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the visible checkout action wording so Portal billing renders `Opening checkout...`, `Open checkout link`, `Start checkout`, and `Resume checkout` instead of `provider checkout` phrasing.
   - `Priority Score: 191`
   - highest commercialization gain with a bounded display-only surface and no runtime risk

2. Leave `provider checkout` visible because the launch logic still targets provider-backed attempts.
   - `Priority Score: 39`
   - rejected because Step 06 still requires the tenant-facing billing workbench to speak in product action language rather than backend launch mechanics

3. Rename provider-launch action contracts across the stack.
   - `Priority Score: 42`
   - rejected because it would widen the slice into contract churn without solving a runtime bug

### Chosen Action

Action 1 was selected because it removes a visible commercialization leak immediately while preserving the already-verified payment-attempt composition, provider launch decisions, and Portal billing runtime behavior.

## Root Cause Summary

### 1. The Checkout Workbench Still Exposed Provider-Launch Jargon

After the earlier billing vocabulary slices, the checkout workbench still rendered action copy such as `Launching provider checkout...`, `Open provider checkout`, and `Resume provider checkout`. That wording described infrastructure posture rather than the tenant-facing checkout action being performed.

### 2. The Runtime Facts Were Already Correct

The issue was not missing checkout URLs, broken retry logic, or incorrect provider launch routing. The billing page already reopened the same existing checkout, created a retry attempt when needed, and started the first attempt when no prior attempt existed. The gap was strictly tenant-facing action wording.

### 3. Shared I18n Coverage Was Incomplete For The Replacement

The new checkout-action wording needed to exist in both the billing page and the shared Portal i18n source contract. The `zh-CN` message catalog also needed matching entries so the Portal surface would not fall back to English.

## Implemented Fixes

- replaced the in-flight status `Launching provider checkout...` with `Opening checkout...`
- replaced the action labels `Open provider checkout`, `Launch provider checkout`, and `Resume provider checkout` with `Open checkout link`, `Start checkout`, and `Resume checkout`
- replaced the first-attempt guidance `Launch the first provider checkout now.` with `Start the first checkout now.`
- updated shared Portal i18n and `zh-CN` translations so the new checkout-action vocabulary is fully registered
- tightened the Portal billing-i18n, product, and commercial-api proof lanes so the new wording is required and the retired provider-checkout wording is blocked

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
  - failed after the red-first test update because the shared Portal i18n source contract still registered the old provider-checkout wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered the old provider-checkout action language
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed after the red-first test update because the billing page source still contained `Launch provider checkout` and `Resume provider checkout`

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- the Portal billing checkout workbench no longer exposes `provider checkout` wording as tenant-facing action copy
- shared Portal i18n and `zh-CN` now carry the new direct checkout-action wording required by the active checkout workbench
- the focused proof lanes now guard this checkout-action vocabulary contract against regression

### Still Open

- other low-level billing wording outside the current checkout-action cluster may still need future productization
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, or architecture ownership boundaries

## Next Slice Recommendation

1. Continue auditing whether tenant-facing billing surfaces still leak `payment attempt` or other provider-oriented implementation terms.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized Portal checkout-action baseline.
