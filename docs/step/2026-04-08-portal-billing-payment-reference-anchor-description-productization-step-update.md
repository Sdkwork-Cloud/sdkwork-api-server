# 2026-04-08 Portal Billing Payment Reference Anchor Description Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by removing the active tenant-facing `{reference} anchors ...` wording from the checkout detail surface, so the payment reference reads like product guidance instead of internal implementation language.

## Closed In This Slice

- replaced the active checkout detail wording with `{reference} is the current {provider} / {channel} payment reference for this order.`
- removed the older `{reference} anchors the current {provider} / {channel} payment method for this order.` wording from both active billing page render paths and the shared Portal i18n source contract
- updated shared Portal i18n and `zh-CN` translations so the new payment-reference wording localizes cleanly
- tightened Portal billing, product, and commercial-api proof so the new payment-reference wording is required and the retired `anchors` wording is blocked

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- the Portal billing page still derives checkout presentation from the same order, provider, and channel facts
- no billing repository composition, checkout decision logic, or commerce payloads changed
- no backend billing or commerce contract changed in this slice

### Only The Payment-Reference Description Changed

- the checkout detail surface no longer exposes `anchors` implementation wording on the active tenant-facing surface
- the slice changed display copy, shared i18n registration, and localized translation only

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another implementation-style phrase from the tenant-facing Portal billing workbench
- improves Step 06 `8.6` evidence by making the checkout reference description read as direct product language
- does not close Step 06 globally because other tenant-facing billing wording may still need future productization
- does not require `docs/架构/*` writeback because this iteration changes presentation copy, i18n coverage, and proof only; it does not change backend contracts, route authority, or ownership boundaries
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
    - full Portal Node suite returned `224 / 224` passing

## Remaining Follow-Up

1. Continue auditing the Portal billing workbench for any remaining tenant-facing finance wording that still leaks implementation terminology.
2. Keep future billing presentation fixes bounded to copy, i18n, and proof unless a real backend contract blocker appears.
3. Continue Step 06 Portal commercialization closure until the billing finance surfaces read as a product workbench end to end.
