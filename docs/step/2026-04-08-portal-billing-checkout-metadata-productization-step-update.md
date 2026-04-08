# 2026-04-08 Portal Billing Checkout Metadata Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by productizing the remaining checkout-method metadata labels in the billing workbench, so the default tenant surface stops exposing internal payment vocabulary such as operator actions, webhook phrasing, and refund-support terminology.

## Closed In This Slice

- renamed checkout method metadata labels from `Operator action`, `Webhook`, `Webhook verification`, `Refund support`, and `Partial refund` to `Manual action`, `Provider events`, `Event signature`, `Refund coverage`, and `Partial refunds`
- updated the Portal billing checkout method cards so the tenant-facing metadata now reads as productized checkout guidance without changing any payment capability flags or action routing
- extended shared Portal i18n and `zh-CN` coverage for the new checkout metadata terminology
- tightened the related Portal payment-rails and billing-i18n proof so the retired metadata wording now fails source-contract verification

## Runtime / Display Truth

### Checkout Method Capabilities Stayed The Same

- checkout methods still expose the same action, session reference, signature, refund, and partial-refund facts
- only the tenant-facing labels changed in this slice
- no payment method capability, provider launch path, refund behavior, or event replay behavior changed

### The Workbench Boundary Is Clearer

- the checkout workbench still shows the same metadata fields on each checkout method card
- the new wording presents those fields as commercial checkout metadata instead of internal integration vocabulary
- simulation, manual settlement, and provider handoff behavior remain unchanged

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another small set of internal payment terms from the default Portal billing workbench
- improves Step 06 `8.6` evidence by aligning checkout metadata labels with a tenant-facing billing vocabulary
- does not close Step 06 globally because broader bridge and sandbox behavior still remains in the billing runtime
- does not require `docs/架构/*` writeback because this slice changes Portal presentation copy, i18n coverage, and proof only; it does not change backend contracts, route authority, or ownership boundaries
- `97` is satisfied through truthful no-writeback classification for this iteration

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## Remaining Follow-Up

1. Continue reducing remaining operator-oriented or integration-oriented payment vocabulary from Portal billing without inventing frontend-only payment behavior.
2. Keep future billing presentation changes tied to the real formal checkout and payment-attempt path.
3. Continue Step 06 Portal commercialization closure until the billing workbench reads as a tenant product surface end to end.
