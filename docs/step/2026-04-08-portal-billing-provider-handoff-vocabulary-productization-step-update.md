# 2026-04-08 Portal Billing Provider Handoff Vocabulary Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by productizing the remaining tenant-facing `Provider handoff` wording in the billing workbench, so the surface speaks in `Checkout access` language instead of exposing the runtime `provider_handoff` action concept directly.

## Closed In This Slice

- renamed the tenant-facing checkout action label from `Provider handoff` to `Checkout access`
- rewrote the payment-attempt and payment-method summary descriptions so they now describe `checkout access` instead of `provider handoff`
- aligned shared Portal i18n coverage with the new checkout-access wording
- aligned `zh-CN` translations with the new checkout-access terminology
- updated Portal billing i18n and product proof so the retired `Provider handoff` wording now fails verification

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- the runtime action enum, service logic, and repository flow still use the same `provider_handoff` semantics internally
- checkout launch, retry, reopen, and payment-attempt behavior remain unchanged
- only visible tenant-facing labels and descriptions changed in this slice

### The Billing Surface Now Speaks In Product Terms

- billing still shows the same provider checkout capability and attempt context
- that capability is now described as `Checkout access` on the tenant surface
- the slice keeps the real provider checkout path intact while removing runtime action jargon from the product copy

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another runtime-facing vocabulary leak from Portal billing
- improves Step 06 `8.6` evidence by aligning checkout action labels and related explanatory copy with product-facing billing language
- does not close Step 06 globally because replay action wording and broader billing/runtime bridge posture still remain elsewhere
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

1. Continue auditing Portal billing replay action wording such as `Replaying provider settlement/failure/cancellation...`.
2. Keep future billing copy changes bounded to tenant-facing vocabulary unless a real contract blocker appears.
3. Continue Step 06 Portal commercialization closure until the billing workbench reads as a tenant product surface end to end.
