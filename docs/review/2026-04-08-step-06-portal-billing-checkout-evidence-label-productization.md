# 2026-04-08 Step 06 Portal Billing Checkout Evidence Label Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by productizing the remaining low-level checkout-evidence labels inside Portal billing, without changing payment runtime contracts, checkout-method payloads, or backend ownership boundaries.

Execution boundary:

- keep `session_reference` and `qr_code_payload` source fields intact
- keep billing repository and service contracts unchanged
- keep checkout-method selection, provider checkout launch, and payment outcome simulation behavior unchanged
- do not introduce new backend payment routes, new billing panels, or new runtime state

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `checkout-evidence-label-productization`
- Previous mode: `verification-method-display-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the remaining checkout-evidence labels so Portal billing renders `Checkout reference` and `QR code content` instead of low-level transport wording.
   - `Priority Score: 188`
   - highest commercialization gain with a bounded display-only surface and no runtime risk

2. Leave the existing labels visible because they reflect backend field semantics precisely.
   - `Priority Score: 41`
   - rejected because Step 06 still requires the tenant-facing billing surface to read like a product workbench rather than a payload-inspection panel

3. Rename the underlying `session_reference` and `qr_code_payload` contracts across the stack.
   - `Priority Score: 49`
   - rejected because it would widen the slice into contract churn without solving a runtime bug

### Chosen Action

Action 1 was selected because it closes a visible commercialization leak immediately while preserving the already-verified checkout-method payloads, billing service composition, and payment runtime behavior.

## Root Cause Summary

### 1. The Billing Workbench Still Exposed Low-Level Evidence Labels

The previous verification-label slice made the values more readable, but the evidence card still labeled the underlying rows as `Session reference` and `QR payload`. Those labels described payload mechanics rather than tenant-facing checkout evidence.

### 2. The Runtime Facts Were Already Correct

The issue was not missing checkout evidence or broken billing composition. The billing page already received the correct canonical `session_reference` and `qr_code_payload` fields. The gap was strictly tenant-facing display language quality on the evidence card.

### 3. Shared I18n And Proof Needed To Move With The Display Labels

The new evidence wording needed to exist in the billing page source, shared i18n registry, and `zh-CN` messages, while proof needed to verify the new vocabulary and prevent regression back to the retired low-level labels.

## Implemented Fixes

- replaced the evidence row label `Session reference` with `Checkout reference` in the Portal billing page
- replaced the QR evidence label `QR payload` with `QR code content` in the Portal billing page
- updated the shared Portal i18n registry so the new evidence labels are part of the active source contract
- updated the `zh-CN` translation map so the new evidence labels are registered for Simplified Chinese
- tightened the billing i18n, product, and payment-rails proof lanes so the new labels are required and the retired low-level labels are now blocked

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
  - failed after the red-first test update because shared Portal i18n did not yet register `Checkout reference` and `QR code content`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered `Session reference` and `QR payload`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
  - failed after the red-first test update because the page source and `zh-CN` messages still contained the retired low-level evidence labels

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- the billing workbench no longer labels checkout evidence with `Session reference` or `QR payload`
- shared Portal i18n and `zh-CN` now carry the product-facing checkout-evidence labels required by the billing surface
- the billing i18n, product, and payment-rails proof lanes now guard this checkout-evidence label contract

### Still Open

- other low-level billing metadata outside these two labels may still need future productization
- unknown future checkout evidence additions may require their own display-language review before they are tenant-facing
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, component ownership, or architecture contracts

## Next Slice Recommendation

1. Continue auditing whether any remaining checkout-method evidence still leaks raw transport or strategy detail into the billing workbench.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more readable checkout-evidence baseline.
