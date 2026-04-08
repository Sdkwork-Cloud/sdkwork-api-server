# 2026-04-08 Portal Billing Provider Checkout Action Vocabulary Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by removing the remaining tenant-facing `provider checkout` action wording from the checkout workbench, so the Portal surface speaks in direct checkout language instead of exposing provider-oriented launch jargon.

## Closed In This Slice

- replaced the in-flight status `Launching provider checkout...` with `Opening checkout...`
- replaced the action labels `Open provider checkout`, `Launch provider checkout`, and `Resume provider checkout` with `Open checkout link`, `Start checkout`, and `Resume checkout`
- replaced the first-attempt guidance `Launch the first provider checkout now.` with `Start the first checkout now.`
- updated shared Portal i18n and `zh-CN` translations so the new checkout-action vocabulary localizes cleanly
- tightened Portal billing, product, and commercial-api proof so the new checkout-action wording is required and the retired provider-checkout wording is blocked

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- the underlying `payment_attempt`, provider-launch decision logic, checkout URL sourcing, and `provider_handoff` runtime action stayed unchanged
- reopening an existing checkout, creating a retry attempt, and starting a first attempt still follow the same launch-decision branches as before
- no billing backend route, transport contract, or checkout execution path changed in this slice

### The Checkout Workbench Now Reads Like Product Language

- the tenant-facing checkout workbench now describes user actions as opening, starting, and resuming checkout instead of exposing provider-launch terminology
- the slice improves billing-action readability without mutating payment-attempt state, launch-decision composition, or provider checkout behavior structurally

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another visible low-level billing term from the Portal checkout workbench
- improves Step 06 `8.6` evidence by aligning the checkout action surface with product checkout vocabulary
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

1. Continue auditing the Portal billing checkout and history workbench for any remaining tenant-facing payment-attempt or provider-oriented jargon.
2. Keep future billing presentation fixes bounded to copy, i18n, and proof unless a real backend contract blocker appears.
3. Continue Step 06 Portal commercialization closure until the billing finance surfaces read as a product workbench end to end.
