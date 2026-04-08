# 2026-04-08 Portal Billing Payment Attempt Vocabulary Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by removing the remaining tenant-facing `payment attempt` wording from the checkout workbench, so the Portal surface speaks in checkout-attempt language instead of exposing payment-attempt implementation terminology.

## Closed In This Slice

- replaced `payment attempt` / `payment attempts` wording with `checkout attempt` / `checkout attempts` across the tenant-facing checkout workbench
- replaced retry and first-attempt guidance so the workbench now says `fresh checkout attempt` and `No {provider} checkout attempt exists yet`
- replaced the failed-payment summary detail so it now describes checkout attempts that closed on the failure path
- updated shared Portal i18n and `zh-CN` translations so the new checkout-attempt vocabulary localizes cleanly
- tightened Portal billing, product, and commercial-api proof so the new checkout-attempt wording is required and the retired payment-attempt wording is blocked from the tenant-facing surface

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- the underlying `payment_attempt_id`, `payment_attempts` payload shape, repository composition, and formal payment-attempt launch path stayed unchanged
- creating retry attempts, starting first attempts, and rendering attempt records still use the same canonical backend payment-attempt flow as before
- no billing backend route, transport contract, or checkout execution path changed in this slice

### The Checkout History Workbench Now Reads Like Product Language

- the tenant-facing checkout workbench now describes these records as checkout attempts rather than payment attempts
- the slice improves billing readability without mutating payment-attempt state, canonical identifiers, or formal checkout behavior structurally

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another visible low-level billing term from the Portal checkout workbench
- improves Step 06 `8.6` evidence by aligning the checkout-history surface with direct checkout vocabulary
- does not close Step 06 globally because other low-level billing wording may still require future productization
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
    - full Portal Node suite returned `221 / 221` passing

## Remaining Follow-Up

1. Continue auditing the Portal billing workbench for any remaining tenant-facing provider-confirmation or refund-history jargon.
2. Keep future billing presentation fixes bounded to copy, i18n, and proof unless a real backend contract blocker appears.
3. Continue Step 06 Portal commercialization closure until the billing finance surfaces read as a product workbench end to end.
