# 2026-04-08 Portal Billing Refund History Description Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by productizing the tenant-facing refund-history panel description, so the finance workbench explains refund evidence in product language instead of exposing closed-loop/operator phrasing.

## Closed In This Slice

- replaced the refund-history panel description with `Refund history keeps completed refund outcomes, payment method evidence, and the resulting order status visible without reopening each order.`
- removed the older `closed-loop refund outcomes` and `verify provider, checkout reference, and final order state` wording from the tenant-facing billing page
- registered the new refund-history description in shared Portal i18n and added matching `zh-CN` coverage
- tightened Portal billing, product, and commercial-api proof so the new refund-history wording is required and the retired wording is blocked from the Portal source contract

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- the refund-history table, payment rows, refund outcomes, and billing repository composition stayed unchanged
- no refund backend route, payment contract, order-status transition, or finance computation changed in this slice
- the slice changed display copy, shared i18n registration, and localized translation only

### The Refund-History Panel Now Reads Like Product Copy

- the tenant-facing billing page now describes refund history as a visible finance evidence surface rather than a closed-loop operator review tool
- the new wording keeps the panel focused on completed refund outcomes, payment method evidence, and resulting order status without changing how those facts are loaded

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another low-level billing phrase from the Portal finance workbench
- improves Step 06 `8.6` evidence by making the refund-history panel read as product-facing finance guidance
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

1. Continue auditing the Portal billing workbench for any remaining tenant-facing refund or finance wording that still leaks low-level implementation language.
2. Keep future billing presentation fixes bounded to copy, i18n, and proof unless a real backend contract blocker appears.
3. Continue Step 06 Portal commercialization closure until the billing finance surfaces read as a product workbench end to end.
