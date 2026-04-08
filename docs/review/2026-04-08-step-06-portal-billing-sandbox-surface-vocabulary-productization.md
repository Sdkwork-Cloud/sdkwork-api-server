# 2026-04-08 Step 06 Portal Billing Sandbox Surface Vocabulary Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by replacing the remaining tenant-facing sandbox title, selector, and verification labels that still exposed low-level `event`, `target`, and `signature` wording, without changing payment runtime contracts, sandbox behavior, or backend ownership boundaries.

Execution boundary:

- keep the existing `sendBillingPaymentEvent` runtime path intact
- keep sandbox target selection, `checkout_method_id`, and `event_type` behavior unchanged
- keep `webhook_verification` data sourcing unchanged
- do not change billing repository contracts, service decisions, or provider callback processing behavior
- do not introduce new backend payment routes, new billing panels, or new runtime state

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `sandbox-surface-vocabulary-productization`
- Previous mode: `payment-outcome-sandbox-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the remaining sandbox title, selector, and verification labels so Portal billing speaks in payment-outcome, sandbox-method, and verification language.
   - `Priority Score: 181`
   - highest commercialization gain with a bounded copy-only surface and no runtime risk

2. Leave the `event` / `target` / `signature` wording in place because the sandbox is inherently technical.
   - `Priority Score: 49`
   - rejected because Step 06 still requires the tenant-facing billing surface to read like a product workbench even when sandbox posture remains visible

3. Rename runtime `webhook_verification` and callback-backed payment contracts to match the new UI copy.
   - `Priority Score: 54`
   - rejected because it would widen the slice into contract churn without solving a runtime bug

### Chosen Action

Action 1 was selected because it closes a visible commercialization leak immediately while preserving the already-verified sandbox selection path, payment outcome application behavior, and backend callback semantics.

## Root Cause Summary

### 1. The Billing Sandbox Still Exposed Low-Level Integration Nouns

The billing workbench had already been partially productized, but the following tenant-facing sandbox surfaces still used low-level wording directly:

- the sandbox title `Payment event sandbox`
- the selector label `Event target`
- the selector placeholder `Choose event target`
- the active status sentence `{provider} is the active sandbox target on {channel}.`
- the checkout-method evidence row label `Event signature`

That wording described the surface as an integration control panel instead of a product-facing billing sandbox.

### 2. The Runtime Facts Were Already Correct

The issue was not missing sandbox state or broken provider targeting. The billing page already targeted the correct callback-backed payment method, emitted the correct `event_type`, and rendered the same `webhook_verification` value from canonical checkout-method presentation. The gap was strictly presentation-language quality on the tenant-facing workbench.

### 3. Shared Proof Needed To Move With The New Product Vocabulary

The old wording existed across the billing page, shared i18n registry, `zh-CN` translations, and the broader payment-rails proof lane. This slice therefore needed to:

- replace the old sandbox vocabulary in the billing page source
- replace the same wording in shared i18n and `zh-CN`
- tighten the billing i18n and product proof lanes so the old terms fail regression checks
- align the broader payment-rails proof lane with the new sandbox-surface vocabulary

## Implemented Fixes

- updated the Portal billing sandbox title from `Payment event sandbox` to `Payment outcome sandbox`
- updated the selector label and placeholder to:
  - `Sandbox method`
  - `Choose sandbox method`
- updated the active sandbox sentence to:
  - `Payment outcomes will use {provider} on {channel}.`
- updated the checkout-method evidence label from `Event signature` to `Verification method`
- updated the shared Portal i18n registry and `zh-CN` messages so the new sandbox-surface vocabulary is registered consistently
- updated `portal-payment-rails.test.mjs` alongside the billing-focused proof lanes so the broader product proof now matches current product wording

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
- `apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed after the red-first test update because the shared i18n registry still exposed the retired sandbox vocabulary
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered `Payment event sandbox`, `Event target`, and `Event signature`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
  - failed after the red-first test update because the billing page and `zh-CN` messages still required the old verification and sandbox labels

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- the billing sandbox no longer exposes `Payment event sandbox`, `Event target`, `Choose event target`, or `Event signature` on the tenant-facing surface
- the active sandbox status message now explains the selected provider/channel in product-facing outcome language instead of target jargon
- shared Portal i18n, `zh-CN`, and the broader payment-rails proof lane now guard against regression back to the retired sandbox-surface vocabulary

### Still Open

- raw `webhook_verification` values still render directly and may need friendlier display mapping in a future slice
- sandbox posture intentionally remains visible because payment simulation still exists as a bounded operator-supporting tenant surface
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, component ownership, or architecture contracts

## Next Slice Recommendation

1. Continue auditing whether raw verification-strategy values in the billing workbench should be mapped to more product-friendly labels.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized sandbox-surface baseline.
