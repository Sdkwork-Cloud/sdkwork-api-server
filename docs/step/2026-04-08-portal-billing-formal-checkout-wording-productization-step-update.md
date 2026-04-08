# 2026-04-08 Portal Billing Formal Checkout Wording Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by removing active tenant-facing `formal checkout` wording from the billing workbench, so the checkout guidance and launch status read like product language instead of internal implementation vocabulary.

## Closed In This Slice

- replaced the payment-method panel description with `Checkout workbench keeps checkout access, selected reference, and payable price aligned under one payment method.`
- replaced the fallback guidance with `No checkout guidance is available for this order yet.`
- replaced the provider launch status copy with `{targetName} now uses the {provider} checkout launch path.`
- replaced the missing-link status copy with `{targetName} created a {provider} checkout attempt, but no checkout link was returned.`
- updated shared Portal i18n, `zh-CN` translations, and focused Portal proof so the new checkout wording is required and the retired `formal` wording is blocked

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- the Portal billing page still loads the same order, payment-method, checkout-session, and payment-attempt facts
- no repository composition, checkout launch decision logic, or provider handoff behavior changed
- no backend billing or commerce contract changed in this slice

### Only Tenant-Facing Checkout Copy Changed

- the billing workbench no longer exposes `formal checkout` language in the active payment-method description, fallback guidance, or provider-launch status messaging
- the status copy now says `checkout link` instead of `checkout URL` on the tenant-facing surface
- the slice changed display copy, shared i18n registration, and localized translation only

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another internal implementation term from the tenant-facing Portal billing workbench
- improves Step 06 `8.6` evidence by making checkout guidance and launch messaging read as direct product language
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
    - full Portal Node suite returned `221 / 221` passing

## Remaining Follow-Up

1. Continue auditing the Portal billing workbench for any remaining tenant-facing finance wording that still leaks internal implementation terminology.
2. Keep future billing presentation fixes bounded to copy, i18n, and proof unless a real backend contract blocker appears.
3. Continue Step 06 Portal commercialization closure until the billing finance surfaces read as a product workbench end to end.
