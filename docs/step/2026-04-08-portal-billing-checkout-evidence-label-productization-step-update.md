# 2026-04-08 Portal Billing Checkout Evidence Label Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by productizing the remaining low-level checkout-evidence labels inside the billing workbench, so the tenant-facing surface says `Checkout reference` and `QR code content` instead of `Session reference` and `QR payload`.

## Closed In This Slice

- replaced the checkout-method evidence row label `Session reference` with `Checkout reference`
- replaced the QR evidence label `QR payload` with `QR code content`
- aligned shared Portal i18n, `zh-CN`, and billing proof lanes with the new checkout-evidence label vocabulary

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- `session_reference` still comes from the same canonical checkout-method payload
- `qr_code_payload` still comes from the same canonical checkout-method payload when QR checkout evidence exists
- repository, service, and transport contracts stayed unchanged in this slice
- only the Portal billing page display labels, shared i18n coverage, and proof changed

### The Workbench Now Reads Like A Product Surface

- checkout-method evidence still shows the same underlying provider facts
- the billing page now presents those facts through more product-facing evidence labels instead of transport-oriented wording
- the slice improves tenant-facing readability without mutating source values or payment flow behavior

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another pair of low-level checkout-evidence wording leaks from Portal billing
- improves Step 06 `8.6` evidence by aligning checkout-method evidence labels with product-facing billing language
- does not close Step 06 globally because other low-level billing metadata may still require future productization
- does not require `docs/架构/*` writeback because this iteration changes presentation copy, i18n coverage, and proof only; it does not change backend contracts, route authority, or ownership boundaries
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

1. Continue auditing whether any remaining checkout-method evidence still leaks other low-level transport or strategy wording into the tenant-facing workbench.
2. Keep future billing presentation fixes bounded to tenant-facing language unless a real backend contract blocker appears.
3. Continue Step 06 Portal commercialization closure until the billing workbench reads as a tenant product surface end to end.
