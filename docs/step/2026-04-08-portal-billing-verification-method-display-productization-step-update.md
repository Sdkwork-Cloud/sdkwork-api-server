# 2026-04-08 Portal Billing Verification Method Display Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by productizing the remaining raw verification-strategy values shown inside checkout-method evidence, so the tenant-facing billing workbench shows readable verification-method labels instead of raw strategy codes.

## Closed In This Slice

- kept the `Verification method` row label introduced in the previous slice, but replaced raw strategy values with product-readable labels at render time
- mapped `manual` to `Manual confirmation`
- mapped `webhook` and `webhook_signed` to `Signed callback check`
- mapped `stripe_signature`, `alipay_rsa_sha256`, and `wechatpay_rsa_sha256` to readable provider-specific verification labels
- aligned shared Portal i18n, `zh-CN`, and billing proof lanes with the new verification-method display vocabulary

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- `webhook_verification` still comes from the same canonical checkout-method payload
- repository, service, and transport contracts still carry the original raw strategy values unchanged
- only the Portal billing page display layer, shared i18n coverage, and proof changed in this slice

### The Workbench Now Renders Human-Readable Verification Labels

- checkout-method evidence still reflects the same underlying verification strategy truth
- the billing page now formats that truth into readable tenant-facing labels instead of exposing raw values such as `stripe_signature`
- the slice keeps the real verification metadata intact while making the checkout-method evidence card read like a product surface

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another low-level strategy-code leak from Portal billing
- improves Step 06 `8.6` evidence by aligning checkout-method verification evidence with product-facing language
- does not close Step 06 globally because other low-level billing metadata may still require future productization
- does not require `docs/架构/*` writeback because this slice changes Portal presentation copy, i18n coverage, and proof only; it does not change backend contracts, route authority, or ownership boundaries
- `97` is satisfied through truthful no-writeback classification for this iteration

## Verification

- red first:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- green:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
  - `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
    - full Portal Node suite returned `221 / 221` passing

## Remaining Follow-Up

1. Continue auditing whether any remaining checkout-method evidence still leaks other raw transport or strategy values directly into the tenant-facing workbench.
2. Keep future billing presentation fixes bounded to tenant-facing language unless a real backend contract blocker appears.
3. Continue Step 06 Portal commercialization closure until the billing workbench reads as a tenant product surface end to end.
