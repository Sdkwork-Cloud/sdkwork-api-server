# 2026-04-08 Portal Billing Payment Update Reference Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by removing the remaining tenant-facing `Provider event` label from the payment-history workbench, so the Portal surface speaks in billing reference language instead of exposing provider-event jargon.

## Closed In This Slice

- replaced the payment-history table header `Provider event` with `Payment update reference`
- added the new shared Portal i18n source key for `Payment update reference`
- replaced the `zh-CN` translation entry so the new payment-history label localizes cleanly
- tightened Portal billing, product, and i18n proof so the new label is required and the retired `Provider event` wording is blocked

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- the underlying `provider_event_id` field, payment-history row shape, and repository/service composition stayed unchanged
- payment-history rows still display the same canonical value when it exists and the same `Not recorded` fallback when it does not
- no billing runtime, transport, or backend contract changed in this slice

### The Payment History Workbench Now Reads Like Product Language

- the tenant-facing payment-history table now presents the external outcome identifier as a payment update reference instead of an internal provider event
- the slice improves finance readability without mutating how payment events are stored, queried, or rendered structurally

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another visible low-level billing term from the Portal payment-history surface
- improves Step 06 `8.6` evidence by aligning the payment-history workbench with billing-product vocabulary
- does not close Step 06 globally because other low-level billing wording may still require future productization
- does not require `docs/架构/*` writeback because this iteration changes presentation copy, i18n coverage, and proof only; it does not change backend contracts, route authority, or ownership boundaries
- `97` is satisfied through truthful no-writeback classification for this iteration

## Verification

- red first:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- green:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
    - full Portal Node suite returned `221 / 221` passing

## Remaining Follow-Up

1. Continue auditing the Portal billing workbench for any remaining tenant-facing payment-history or refund-history jargon.
2. Keep future billing presentation fixes bounded to copy, i18n, and proof unless a real backend contract blocker appears.
3. Continue Step 06 Portal commercialization closure until the billing finance surfaces read as a product workbench end to end.
