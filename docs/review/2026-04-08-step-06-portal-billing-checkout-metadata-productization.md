# 2026-04-08 Step 06 Portal Billing Checkout Metadata Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by cleaning up the remaining checkout-method metadata terminology in billing, without changing payment runtime contracts, capability flags, or backend ownership boundaries.

Execution boundary:

- keep the existing checkout-method metadata fields and values intact
- keep formal order, payment-method, and payment-attempt truth primary
- do not change provider launch, settlement, refund, or event replay behavior
- do not introduce new backend payment routes, new billing surfaces, or new capability calculations

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `checkout-metadata-productization`
- Previous mode: `settlement-and-sandbox-posture-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the remaining checkout metadata labels on Portal billing method cards so the surface reads as commercial checkout guidance instead of operator or webhook terminology.
   - `Priority Score: 169`
   - highest commercialization gain with the smallest copy-only surface and no runtime risk

2. Remove the underlying metadata fields from the checkout method cards entirely.
   - `Priority Score: 82`
   - rejected because the metadata still carries useful user-facing checkout facts and removing it would widen the slice into information architecture changes

3. Leave the existing labels in place because they are technically accurate.
   - `Priority Score: 37`
   - rejected because Step 06 still requires the visible tenant-facing billing workbench to read like a commercial product, not an internal payment integration console

### Chosen Action

Action 1 was selected because it closes a visible commercialization gap immediately while preserving the already-verified payment behavior and checkout metadata structure.

## Root Cause Summary

### 1. Checkout Method Cards Still Exposed Internal Integration Vocabulary

The billing workbench already showed useful checkout metadata:

- action posture
- event callback capability
- verification signature
- refund availability
- partial refund availability

but the labels still used terms such as `Operator action`, `Webhook`, `Webhook verification`, `Refund support`, and `Partial refund`. That wording made the default tenant workbench read like an internal gateway or operator console.

### 2. The Runtime Facts Were Already Correct

The issue was not missing data or broken payment behavior. The checkout method cards already rendered the right metadata values from canonical checkout methods. The gap was strictly presentation-language quality on the tenant-facing billing surface.

### 3. Shared I18n Truth Needed To Move In Lockstep

Because the Portal i18n layer uses source-contract tests and a shared message registry, the copy change required synchronized updates to:

- the billing page source
- the shared message-key registry
- `zh-CN` translations
- payment-rails and billing-i18n proof lanes

## Implemented Fixes

- updated the Portal billing page to relabel checkout metadata fields as:
  - `Manual action`
  - `Provider events`
  - `Event signature`
  - `Refund coverage`
  - `Partial refunds`
- updated shared Portal i18n registries and `zh-CN` translations for the new checkout metadata vocabulary
- updated source-contract tests so they now:
  - require the new productized metadata wording
  - reject the retired operator / webhook / refund-support phrasing
  - keep the regex boundary strict enough that `Partial refunds` no longer falsely trips the retired `Partial refund` matcher

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
- `apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
  - failed after the red-first test update because billing still rendered `Operator action`, `Webhook`, `Webhook verification`, `Refund support`, and `Partial refund`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed after the red-first test update because the shared Portal i18n registry still exposed the retired checkout metadata wording

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- checkout method cards no longer expose internal operator, webhook, or refund-support wording on the default tenant billing surface
- shared Portal i18n and `zh-CN` now cover the new checkout metadata product copy
- the payment-rails and billing-i18n proof lanes now guard against regression back to the retired metadata terminology

### Still Open

- explicit payment-event sandbox behavior still remains in the billing runtime
- manual settlement bridge behavior still remains when simulation posture permits it
- broader payment-rail and checkout-workbench vocabulary still needs continued Step 06 auditing
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, component ownership, or architecture contracts

## Next Slice Recommendation

1. Continue auditing billing for any remaining copy that still exposes payment-integration or bridge concepts as the default tenant posture.
2. Start from the Portal billing page and preserve runtime behavior while tightening only the visible wording boundary.
3. Continue Step 06 commercialization closure with the next smallest Portal billing surface that still exposes internal payment vocabulary.
