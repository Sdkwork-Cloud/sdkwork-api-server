# 2026-04-08 Portal Billing Fallback Reason Description Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by productizing the tenant-facing fallback-reason description, so the Portal billing workbench explains degraded routing in user-facing language instead of speaking from an operator perspective.

## Closed In This Slice

- replaced the fallback-reason description with `Fallback reasoning stays visible so you can distinguish degraded routing from the preferred routing path.`
- removed the older `operators can distinguish degraded routing from normal preference selection` wording from the tenant-facing billing page and shared Portal i18n source contract
- updated shared Portal i18n and `zh-CN` translations so the new fallback-reason description localizes cleanly
- tightened Portal billing, product, and commercial-api proof so the new fallback-reason wording is required and the retired operator-facing wording is blocked

## Runtime / Display Truth

### Billing Behavior Stayed The Same

- the billing event analytics cards, fallback evidence, and routing counters stayed unchanged
- no routing backend contract, billing event payload, preview logic, or finance computation changed in this slice
- the slice changed display copy, shared i18n registration, and localized translation only

### The Fallback Description Now Uses Tenant-Facing Language

- the tenant-facing billing page now explains fallback reasoning from the user viewpoint rather than an operator review viewpoint
- the new wording keeps the panel focused on degraded routing versus the preferred routing path without changing how those facts are loaded or rendered

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by removing another operator-facing phrase from the Portal billing workbench
- improves Step 06 `8.6` evidence by making routing fallback guidance read as direct product language
- does not close Step 06 globally because other tenant-facing billing copy may still need future productization
- does not require `docs/é‹èˆµç€¯/*` writeback because this iteration changes presentation copy, i18n coverage, and proof only; it does not change backend contracts, route authority, or ownership boundaries
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

1. Continue auditing the Portal billing workbench for any remaining tenant-facing finance or routing wording that still leaks internal terminology.
2. Keep future billing presentation fixes bounded to copy, i18n, and proof unless a real backend contract blocker appears.
3. Continue Step 06 Portal commercialization closure until the billing finance surfaces read as a product workbench end to end.
