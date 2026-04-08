# 2026-04-08 Step 06 Portal Billing Payment Update Reference Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by productizing the payment-history table label that still exposed `Provider event` on the tenant-facing Portal billing surface, without changing payment-history contracts, backend ownership boundaries, or runtime behavior.

Execution boundary:

- keep `provider_event_id`, payment-history row contracts, and repository/service composition unchanged
- keep payment-history rendering structure, sorting, and value sourcing unchanged
- keep Portal billing runtime behavior unchanged
- do not introduce new backend billing routes, history fields, or finance panels

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `payment-update-reference-productization`
- Previous mode: `checkout-session-vocabulary-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the visible payment-history label so Portal billing renders `Payment update reference` instead of `Provider event`.
   - `Priority Score: 187`
   - highest commercialization gain with a bounded display-only surface and no runtime risk

2. Leave `Provider event` visible because it matches the stored field name.
   - `Priority Score: 34`
   - rejected because Step 06 still requires the tenant-facing finance surface to read like a product workbench rather than an internal event log

3. Rename `provider_event_id` and related contracts across the stack.
   - `Priority Score: 45`
   - rejected because it would widen the slice into contract churn without solving a runtime bug

### Chosen Action

Action 1 was selected because it removes a visible commercialization leak immediately while preserving the already-verified payment-history payload, repository composition, and Portal billing runtime behavior.

## Root Cause Summary

### 1. The Payment History Workbench Still Exposed Provider-Event Jargon

After the earlier billing vocabulary slices, the payment-history audit table still rendered `Provider event` as a visible column label. That wording described the storage origin of the identifier rather than the tenant-facing billing meaning of the value.

### 2. The Runtime Facts Were Already Correct

The issue was not missing payment-history data or incorrect event wiring. The billing page already received the correct canonical history rows and still needed to show the same identifier value. The gap was strictly tenant-facing label quality on the finance workbench.

### 3. Shared I18n Coverage Was Incomplete For The Replacement

The new product-facing label needed to exist in both the billing page and the shared Portal i18n source contract. The `zh-CN` message catalog also needed a matching entry so the Portal surface would not fall back to English.

## Implemented Fixes

- replaced the payment-history table header `Provider event` with `Payment update reference`
- added the new `Payment update reference` key to the shared Portal i18n source registry
- replaced the `zh-CN` translation entry with a localized payment-update-reference label
- tightened the Portal payment-history, product, and billing-i18n proof lanes so the new label is required and the retired `Provider event` wording is blocked

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
- `apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
  - failed after the red-first test update because the billing page still rendered `Provider event`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still lacked the new `Payment update reference` wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed after the red-first test update because the shared Portal i18n source contract did not yet register `Payment update reference`

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- the Portal billing payment-history table no longer exposes `Provider event` as tenant-facing copy
- shared Portal i18n and `zh-CN` now carry the new payment-update-reference wording required by the active billing surface
- the focused proof lanes now guard this payment-history label contract against regression

### Still Open

- other low-level billing wording outside the current payment-history label cluster may still need future productization
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, or architecture ownership boundaries

## Next Slice Recommendation

1. Continue auditing whether payment-history and refund-history surfaces still leak provider-oriented implementation terms into tenant-facing finance views.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized Portal billing history baseline.
