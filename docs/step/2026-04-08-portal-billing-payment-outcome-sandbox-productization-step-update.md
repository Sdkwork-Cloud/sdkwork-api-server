# 2026-04-08 Portal Billing Payment Outcome Sandbox Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by productizing the remaining tenant-facing provider-event and replay wording inside the billing sandbox surface, so the Portal workbench speaks in payment-outcome language instead of provider-event mechanics.

## Closed In This Slice

- renamed the sandbox capability badge from `Provider events` to `Payment outcomes`
- rewrote the sandbox guidance sentence from provider-event replay wording to payment-outcome wording
- rewrote the in-progress replay status messages from `Replaying ...` to `Applying ... outcome ...`
- rewrote the sandbox action buttons from `Replay ... event` to `Apply ... outcome`
- aligned shared Portal i18n and `zh-CN` translations with the new payment-outcome vocabulary
- updated the billing outcome proof lane in `portal-payment-rails.test.mjs` so it validates the new product wording instead of the retired provider-event label

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- the sandbox still uses the same `sendBillingPaymentEvent` runtime path
- `event_type`, `provider_event_id`, `checkout_method_id`, and provider selection behavior remain unchanged
- only visible tenant-facing copy and source-contract proof changed in this slice

### The Sandbox Now Speaks In Product Terms

- the same sandbox surface still lets operators trigger settled, failed, and canceled billing outcomes for the active target
- those actions are now described as applying payment outcomes instead of replaying provider events
- the slice keeps the real billing sandbox mechanics intact while removing the remaining provider-event jargon from the tenant-facing workbench

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another cluster of integration-oriented provider-event wording from Portal billing
- improves Step 06 `8.6` evidence by aligning sandbox badges, action labels, progress feedback, and supporting copy with product-facing billing language
- does not close Step 06 globally because `Payment event sandbox`, `Event target`, and other sandbox-specific terminology still remain elsewhere
- does not require `docs/架构/*` writeback because this slice changes Portal presentation copy, i18n coverage, and proof only; it does not change backend contracts, route authority, or ownership boundaries
- `97` is satisfied through truthful no-writeback classification for this iteration

## Verification

- red first:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- green:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
  - first full-suite run surfaced one stale proof lane in `apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs` at `220 / 221`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
    - final full Portal Node suite returned `221 / 221` passing

## Remaining Follow-Up

1. Continue auditing whether `Payment event sandbox` and `Event target` should be further productized or intentionally remain explicit sandbox vocabulary.
2. Keep future billing sandbox copy changes bounded to tenant-facing language unless a real backend contract blocker appears.
3. Continue Step 06 Portal commercialization closure until the billing workbench reads as a tenant product surface end to end.
