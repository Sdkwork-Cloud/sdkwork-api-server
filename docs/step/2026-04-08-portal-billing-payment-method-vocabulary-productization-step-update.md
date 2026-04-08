# 2026-04-08 Portal Billing Payment Method Vocabulary Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by productizing the remaining `rail` vocabulary in the billing workbench, so the default tenant surface speaks in payment-method and sandbox-target language instead of internal payment-routing terminology.

## Closed In This Slice

- renamed the tenant-facing billing `Payment rail` surfaces to `Payment method`
- renamed `Primary rail` to `Primary method`
- renamed the sandbox selector from `Event rail` to `Event target`
- rewrote the remaining workbench, history, and sandbox sentences that still referred to `selected payment rail`, `different payment rail`, `sandbox rail`, or `payment rail evidence`
- added the missing shared Portal i18n key for `Payment method` and aligned `zh-CN` coverage with the new payment-method vocabulary
- updated Portal billing i18n and product proof so the retired `rail` wording now fails verification

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- checkout methods, payment attempts, sandbox actions, and history data still come from the same runtime sources
- only the visible tenant-facing labels and descriptions changed in this slice
- no provider launch path, refund behavior, event replay behavior, or payment-method capability changed

### The Workbench Now Reads As Product Copy

- the billing workbench still presents the same provider/channel facts and sandbox selection state
- those facts are now described as `Payment method`, `Primary method`, and `Event target`
- the slice keeps formal checkout and sandbox posture intact while removing a layer of internal routing jargon from the tenant surface

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another cluster of integration-oriented payment vocabulary from Portal billing
- improves Step 06 `8.6` evidence by aligning checkout summary, status guidance, history copy, and sandbox selection wording with product-facing billing language
- does not close Step 06 globally because manual settlement bridge behavior, provider callback wording in some outcome messages, and broader billing/runtime bridge posture still remain elsewhere
- does not require `docs/架构/*` writeback because this slice changes Portal presentation copy, i18n coverage, and proof only; it does not change backend contracts, route authority, or ownership boundaries
- `97` is satisfied through truthful no-writeback classification for this iteration

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## Remaining Follow-Up

1. Continue auditing Portal billing for any remaining provider-bridge or callback-oriented terms that still leak into tenant-facing copy.
2. Keep future billing copy changes tied to the real formal checkout and payment-attempt path instead of inventing frontend-only abstractions.
3. Continue Step 06 Portal commercialization closure until the billing workbench reads as a tenant product surface end to end.
