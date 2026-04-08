# 2026-04-08 Portal Billing Sandbox Surface Vocabulary Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by productizing the remaining tenant-facing sandbox surface wording that still exposed `event`, `target`, and `signature` jargon, so the billing sandbox speaks in payment-outcome, sandbox-method, and verification language instead of low-level integration terms.

## Closed In This Slice

- renamed the sandbox title from `Payment event sandbox` to `Payment outcome sandbox`
- renamed the sandbox selector from `Event target` / `Choose event target` to `Sandbox method` / `Choose sandbox method`
- renamed the checkout-method evidence label from `Event signature` to `Verification method`
- rewrote the active sandbox sentence from `{provider} is the active sandbox target on {channel}.` to `Payment outcomes will use {provider} on {channel}.`
- aligned shared Portal i18n, `zh-CN`, and the billing proof lanes with the new sandbox-surface vocabulary

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- the sandbox still uses the same `sendBillingPaymentEvent` runtime path
- `providerCallbackMethodId`, `checkout_method_id`, `event_type`, and `webhook_verification` behavior remain unchanged
- only visible tenant-facing copy, shared i18n coverage, and proof changed in this slice

### The Sandbox Surface Now Speaks In Product Terms

- the same simulation surface still applies settlement, failure, and cancellation outcomes for the selected callback-backed payment method
- the same checkout-method card still shows the same verification metadata value, but the row is now labeled as `Verification method`
- the slice keeps the real billing sandbox mechanics intact while removing another cluster of low-level event/target wording from the tenant workbench

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another tenant-visible sandbox vocabulary leak from Portal billing
- improves Step 06 `8.6` evidence by aligning sandbox title, selector, status sentence, and verification label with product-facing billing language
- does not close Step 06 globally because other low-level payment metadata may still need additional productization later
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

1. Continue auditing whether raw `webhook_verification` values should be further mapped into more product-friendly display labels instead of rendering strategy-style strings directly.
2. Keep future billing sandbox copy changes bounded to tenant-facing language unless a real backend contract blocker appears.
3. Continue Step 06 Portal commercialization closure until the billing workbench reads as a tenant product surface end to end.
