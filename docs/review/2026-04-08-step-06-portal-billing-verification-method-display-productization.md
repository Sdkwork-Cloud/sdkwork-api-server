# 2026-04-08 Step 06 Portal Billing Verification Method Display Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by replacing raw checkout-method verification strategy values with readable tenant-facing labels in Portal billing, without changing payment runtime contracts, checkout-method payloads, or backend ownership boundaries.

Execution boundary:

- keep the existing `webhook_verification` field and source data intact
- keep billing repository and service contracts unchanged
- keep checkout-method selection, payment outcome simulation, and provider callback behavior unchanged
- do not introduce new backend payment routes, new billing panels, or new runtime state

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `verification-method-display-productization`
- Previous mode: `sandbox-surface-vocabulary-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the remaining raw verification-strategy values so checkout-method evidence renders readable verification labels instead of raw codes.
   - `Priority Score: 182`
   - highest commercialization gain with a bounded display-only surface and no runtime risk

2. Leave the raw strategy values visible because they are technically precise.
   - `Priority Score: 45`
   - rejected because Step 06 still requires the tenant-facing billing surface to read like a product workbench rather than a transport-debug panel

3. Rename the underlying `webhook_verification` field and upstream strategy values to product labels.
   - `Priority Score: 53`
   - rejected because it would widen the slice into contract churn without solving a runtime bug

### Chosen Action

Action 1 was selected because it closes a visible commercialization leak immediately while preserving the already-verified checkout-method payload, simulation behavior, and backend verification semantics.

## Root Cause Summary

### 1. The Billing Workbench Still Exposed Raw Strategy Codes

The previous slice had already productized the row label to `Verification method`, but the displayed values still came straight from `method.webhook_verification`. That meant the tenant-facing workbench still surfaced raw values such as:

- `manual`
- `webhook`
- `webhook_signed`
- `stripe_signature`
- `alipay_rsa_sha256`
- `wechatpay_rsa_sha256`

That wording described internal verification strategies directly instead of presenting readable verification methods.

### 2. The Runtime Facts Were Already Correct

The issue was not missing verification metadata or broken checkout-method composition. The billing service already carried the correct canonical `webhook_verification` values from commerce checkout methods. The gap was strictly display-language quality on the tenant-facing workbench.

### 3. Shared I18n And Proof Needed To Move With The Display Mapping

The new readable verification labels needed to exist in the billing page source, shared i18n registry, and `zh-CN` messages, while proof needed to verify the readable labels and prevent raw strategy codes from leaking into shared Portal copy.

## Implemented Fixes

- added a display-only `checkoutMethodVerificationLabel(...)` mapping in the Portal billing page
- mapped raw verification strategies to readable labels:
  - `manual` → `Manual confirmation`
  - `webhook` / `webhook_signed` → `Signed callback check`
  - `stripe_signature` → `Stripe signature check`
  - `alipay_rsa_sha256` → `Alipay RSA-SHA256 check`
  - `wechatpay_rsa_sha256` → `WeChat Pay RSA-SHA256 check`
- kept the fallback behavior display-only so unknown values still preserve source truth without changing transport contracts
- updated the shared Portal i18n registry and `zh-CN` messages so the new verification labels are registered consistently
- tightened the billing i18n, product, and payment-rails proof lanes so the readable verification labels are now part of the source contract

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
  - failed after the red-first test update because shared Portal i18n did not yet register the new readable verification labels
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered `method.webhook_verification` directly without the readable verification labels
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
  - failed after the red-first test update because neither the page source nor `zh-CN` messages contained the new verification-method display strings

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- the billing workbench no longer renders raw verification strategy codes directly in the tenant-facing checkout-method evidence row
- shared Portal i18n and `zh-CN` now carry the readable verification labels required by the billing surface
- the billing i18n, product, and payment-rails proof lanes now guard this verification-method display contract

### Still Open

- unknown future verification strategy values still fall back to source-driven display and may need explicit product mappings if they begin to surface in the tenant workbench
- other low-level billing metadata outside the verification row may still need future productization
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, component ownership, or architecture contracts

## Next Slice Recommendation

1. Continue auditing whether any remaining checkout-method evidence still leaks raw strategy or transport detail into the billing workbench.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more readable verification-method baseline.
