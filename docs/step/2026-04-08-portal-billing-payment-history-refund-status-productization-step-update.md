# 2026-04-08 Portal Billing Payment History Refund Status Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by productizing the tenant-facing payment-history panel description, so the Portal finance workbench says `refund status` instead of the more internal `refund closure` wording.

## Closed In This Slice

- replaced the payment-history panel description with `Payment history keeps checkout outcomes, payment method evidence, and refund status visible in one billing timeline.`
- removed the older `refund closure` wording from the tenant-facing billing page and shared Portal i18n source contract
- updated shared Portal i18n and `zh-CN` translations so the new payment-history description localizes cleanly
- tightened Portal billing, product, and commercial-api proof so the new payment-history wording is required and the retired `refund closure` term is blocked

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- the payment-history table, billing repository composition, payment outcomes, and refund evidence stayed unchanged
- no billing backend route, order-state transition, refund contract, or finance computation changed in this slice
- the slice changed display copy, shared i18n registration, and localized translation only

### The Payment-History Panel Now Uses Product Language

- the tenant-facing billing page now refers to `refund status` instead of `refund closure`
- the new wording keeps the panel focused on checkout outcomes, payment method evidence, and refund state without changing how those facts are loaded or rendered

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another low-level finance phrase from the Portal billing workbench
- improves Step 06 `8.6` evidence by making the payment-history panel read as direct product language
- does not close Step 06 globally because other tenant-facing billing copy may still need future productization
- does not require `docs/é‹èˆµç€¯/*` writeback because this iteration changes presentation copy, i18n coverage, and proof only; it does not change backend contracts, route authority, or ownership boundaries
- `97` is satisfied through truthful no-writeback classification for this iteration

## Verification

- red first:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- green:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
    - full Portal Node suite returned `221 / 221` passing

## Remaining Follow-Up

1. Continue auditing the Portal billing workbench for any remaining tenant-facing finance wording that still leaks low-level implementation terminology.
2. Keep future billing presentation fixes bounded to copy, i18n, and proof unless a real backend contract blocker appears.
3. Continue Step 06 Portal commercialization closure until the billing finance surfaces read as a product workbench end to end.
