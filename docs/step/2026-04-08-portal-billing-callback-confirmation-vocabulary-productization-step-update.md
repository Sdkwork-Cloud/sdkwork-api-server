# 2026-04-08 Portal Billing Callback Confirmation Vocabulary Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by productizing the remaining `callback flow` and `provider callback` wording in billing replay outcome feedback, so the tenant surface describes payment confirmation instead of integration callback mechanics.

## Closed In This Slice

- renamed the remaining tenant-facing replay outcome messages from `callback flow` / `provider callback` wording to `payment confirmation`
- updated the provider-specific billing outcome copy for settled, failed, and canceled replay states
- updated the provider-generic shared Portal i18n variants so fallback wording also uses `payment confirmation`
- aligned `zh-CN` translations with the new payment-confirmation terminology
- updated Portal billing i18n and product proof so the retired callback wording now fails verification

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- payment replay execution, provider event handling, settlement state changes, and cancellation behavior still come from the same runtime paths
- only visible tenant-facing outcome copy changed in this slice
- no checkout launch behavior, payment-method capability, provider callback processing, or billing authority changed

### The Billing Surface Now Speaks In Product Terms

- replay outcomes still report the same settled, failed, and canceled state transitions
- those state transitions are now described as happening after `payment confirmation`
- the slice keeps the actual provider-event bridge intact while removing callback-integration jargon from tenant-facing status feedback

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another cluster of provider-integration wording from Portal billing replay feedback
- improves Step 06 `8.6` evidence by aligning replay-result status copy with product-facing billing language
- does not close Step 06 globally because `provider handoff`, replay action wording, and broader billing/runtime bridge posture still remain elsewhere
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
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
    - full Portal Node suite returned `221 / 221` passing

## Remaining Follow-Up

1. Continue auditing Portal billing for remaining provider-bridge wording such as `Provider handoff`.
2. Revisit replay action labels if they still read as provider-event tooling instead of product-facing billing operations.
3. Continue Step 06 Portal commercialization closure until the billing workbench reads as a tenant product surface end to end.
