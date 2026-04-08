# 2026-04-08 Portal Billing Commercial Account Description Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by removing the active tenant-facing `Commercial account exposes canonical balance, hold, and account identity state ...` wording from the billing workbench, so the commercial-account summary reads like product guidance instead of internal implementation vocabulary.

## Closed In This Slice

- replaced the commercial-account panel description with `Commercial account keeps balance, holds, and account identity visible beside the workspace billing posture.`
- removed the older `Commercial account exposes canonical balance, hold, and account identity state ...` wording from the tenant-facing billing page and shared Portal i18n source contract
- updated shared Portal i18n and `zh-CN` translations so the new commercial-account description localizes cleanly
- tightened Portal billing, product, and commercial-api proof so the new commercial-account description is required and the retired `canonical state` wording is blocked

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- the Portal billing page still loads the same commercial-account facts for balance, hold posture, and account identity
- no billing repository composition, finance calculations, or commerce payloads changed
- no backend billing or commerce contract changed in this slice

### Only The Commercial-Account Description Changed

- the commercial-account panel no longer exposes `canonical` and `state` implementation wording on the active tenant-facing surface
- the slice changed display copy, shared i18n registration, and localized translation only

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another internal implementation phrase from the tenant-facing Portal billing workbench
- improves Step 06 `8.6` evidence by making the commercial-account summary read as direct product language
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
    - full Portal Node suite returned `222 / 222` passing

## Remaining Follow-Up

1. Continue auditing the Portal billing workbench for any remaining tenant-facing finance wording that still leaks internal implementation terminology.
2. Keep future billing presentation fixes bounded to copy, i18n, and proof unless a real backend contract blocker appears.
3. Continue Step 06 Portal commercialization closure until the billing finance surfaces read as a product workbench end to end.
