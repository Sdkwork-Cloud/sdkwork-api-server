# 2026-04-08 Step 06 Portal Billing Payment Method Vocabulary Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by replacing the remaining tenant-facing `rail` vocabulary in billing with payment-method and sandbox-target language, without changing payment runtime contracts, action availability, or backend ownership boundaries.

Execution boundary:

- keep the existing checkout, history, and sandbox data model intact
- keep formal order, payment-method, and payment-attempt truth primary
- do not change provider launch, settlement, refund, or event replay behavior
- do not introduce new backend payment routes, new billing panels, or new runtime state

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `payment-method-vocabulary-productization`
- Previous mode: `checkout-metadata-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the remaining `rail` wording across the Portal billing workbench so the visible tenant surface speaks in payment-method and sandbox-target language.
   - `Priority Score: 171`
   - highest commercialization gain with a bounded copy-only surface and no runtime risk

2. Leave the `rail` wording in place because it still describes the provider/channel pairing accurately enough for internal operators.
   - `Priority Score: 39`
   - rejected because Step 06 still requires the tenant-facing billing workbench to read like a commercial product, not an internal payment-routing console

3. Remove the summary and sandbox selector wording entirely instead of renaming it.
   - `Priority Score: 84`
   - rejected because it would widen the slice into information-architecture changes and hide useful checkout facts that remain valuable to tenants

### Chosen Action

Action 1 was selected because it closes a visible commercialization gap immediately while preserving the already-verified payment behavior, workbench layout, and history/sandbox data flow.

## Root Cause Summary

### 1. Billing Still Exposed Internal Routing Vocabulary

The workbench had already been partially productized, but the following tenant-facing surfaces still used `rail` terminology:

- payment-history and refund-history table headers
- checkout guidance and anchor sentences
- failed-payment guidance
- the payment summary panel title and primary fact label
- the sandbox selector and active sandbox status sentence

That wording described the surface as an internal routing console instead of a tenant billing product.

### 2. The Runtime Facts Were Already Correct

The issue was not missing data or broken payment behavior. The billing page already rendered the right provider/channel, reference, attempt, and sandbox facts from canonical checkout detail. The gap was strictly presentation-language quality on the tenant-facing workbench.

### 3. Shared I18n Truth Was Incomplete

The shared i18n registry already covered several related billing strings, but `Payment rail` itself had not been registered as a shared key. This slice needed to:

- replace the old `rail` strings in billing page source
- register the new `Payment method` vocabulary in shared i18n
- align `zh-CN` mappings with the new keys
- keep source-contract tests strict enough to reject regression back to `rail` wording

## Implemented Fixes

- updated the Portal billing page to rename:
  - `Payment rail` -> `Payment method`
  - `Primary rail` -> `Primary method`
  - `Event rail` -> `Event target`
  - `Choose event rail` -> `Choose event target`
- rewrote the remaining checkout, history, and sandbox copy so it now refers to:
  - `selected payment method`
  - `different payment method`
  - `payment method evidence`
  - `active sandbox target`
- added the missing shared i18n key for `Payment method`
- updated `zh-CN` coverage so the new payment-method vocabulary no longer falls back to English in Chinese locale
- tightened product and billing i18n source-contract tests so the retired `rail` terminology now fails proof

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
  - failed after the red-first test update because the billing page and shared i18n registry still exposed `selected payment rail`, `different payment rail`, and related `rail` wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing product surface still rendered `Payment rail` and `Primary rail`

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- the billing workbench no longer exposes `rail` vocabulary on its visible payment summary, status guidance, history copy, or sandbox selector
- shared Portal i18n and `zh-CN` now cover the new payment-method vocabulary consistently
- the billing i18n and product proof lanes now guard against regression back to `Payment rail`, `Primary rail`, and `Event rail`

### Still Open

- explicit sandbox behavior still remains in runtime when simulation posture is enabled
- provider callback wording still remains in some outcome messages outside this vocabulary slice
- manual settlement bridge behavior still remains when simulation posture permits it
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, component ownership, or architecture contracts

## Next Slice Recommendation

1. Continue auditing Portal billing for remaining provider-bridge and callback-oriented phrases that still read like internal integration posture.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized payment-method baseline.
