# 2026-04-08 Step 06 Portal Billing Provider Handoff Vocabulary Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by replacing the remaining tenant-facing `Provider handoff` wording in billing with `Checkout access` language, without changing payment runtime contracts, checkout launch behavior, or backend ownership boundaries.

Execution boundary:

- keep the existing `provider_handoff` runtime enum and launch logic intact
- keep checkout launch, reopen, retry, and payment-attempt behavior unchanged
- do not change billing repository contracts, service decisions, or provider checkout routing
- do not introduce new backend payment routes, new billing panels, or new runtime state

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `provider-handoff-vocabulary-productization`
- Previous mode: `callback-confirmation-vocabulary-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the remaining `Provider handoff` wording so Portal billing describes the action and supporting explanations as `Checkout access`.
   - `Priority Score: 174`
   - highest commercialization gain with a bounded copy-only surface and no runtime risk

2. Leave the `Provider handoff` wording in place because it mirrors the runtime action enum exactly.
   - `Priority Score: 44`
   - rejected because Step 06 still requires the tenant-facing billing surface to read like a product workbench, not an internal action ledger

3. Rename the runtime `provider_handoff` enum and service-layer logic to match the product copy.
   - `Priority Score: 51`
   - rejected because it would widen the slice into contract churn without solving a runtime bug

### Chosen Action

Action 1 was selected because it closes a visible commercialization leak immediately while preserving the already-verified checkout launch path, payment-attempt behavior, and backend action semantics.

## Root Cause Summary

### 1. The Billing Surface Still Exposed A Runtime Action Name

The workbench had already been partially productized, but the following tenant-facing surfaces still used runtime-oriented `Provider handoff` wording directly:

- the checkout action label for `provider_handoff`
- the payment-attempt explanation sentence
- the payment-method summary description

That wording described the surface as an internal action map instead of a tenant-facing checkout product.

### 2. The Runtime Facts Were Already Correct

The issue was not missing data or broken checkout launch behavior. The billing page already rendered the correct provider checkout action, retry posture, and checkout references from canonical commerce detail. The gap was strictly presentation-language quality on the tenant-facing workbench.

### 3. Shared I18n Truth Needed To Move With The Page Copy

The `Provider handoff` wording was shared across the billing page, common i18n registry, and `zh-CN` messages. This slice needed to:

- replace `Provider handoff` in the billing page source
- replace the related descriptive strings in shared i18n
- align `zh-CN` translations with the new `Checkout access` language
- keep source-contract tests strict enough to reject regression back to `Provider handoff`

## Implemented Fixes

- updated the Portal billing page checkout action label from `Provider handoff` to `Checkout access`
- updated the payment-attempt explanation to:
  - `Formal order-scoped payment attempts keep checkout access, retries, and checkout references visible inside the same workbench.`
- updated the payment-method summary description to:
  - `Formal checkout keeps checkout access, selected reference, and payable price aligned under one payment method.`
- updated the shared Portal i18n registry and `zh-CN` messages so the new checkout-access vocabulary is registered consistently
- tightened billing i18n and product proof so the retired `Provider handoff` wording now fails source-contract verification

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
- `apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed after the red-first test update because the shared i18n registry still exposed `Provider handoff`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered `Provider handoff` and the old explanatory sentences

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- the billing workbench no longer exposes `Provider handoff` in tenant-facing checkout labels or the two related explanatory descriptions
- shared Portal i18n and `zh-CN` now describe the same capability through `Checkout access`
- the billing i18n and product proof lanes now guard against regression back to `Provider handoff`

### Still Open

- replay action wording still includes `Replaying provider settlement/failure/cancellation...`
- other provider-event oriented sandbox guidance still remains in billing
- manual settlement bridge behavior still remains when simulation posture permits it
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, component ownership, or architecture contracts

## Next Slice Recommendation

1. Continue auditing Portal billing replay status and action wording for remaining provider-event terminology.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized checkout-access baseline.
