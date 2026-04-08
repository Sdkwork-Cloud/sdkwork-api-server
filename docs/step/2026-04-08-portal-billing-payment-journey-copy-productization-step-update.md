# 2026-04-08 Portal Billing Payment Journey Copy Productization Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by productizing the remaining user-facing payment-journey copy across billing and recharge, so the Portal no longer guides tenants through settlement/operator language where the intended next step is the checkout workbench or a normal checkout completion flow.

## Closed In This Slice

- changed the billing checkout-session guidance from `Open session ... inspect the payment rail` to `Open the checkout workbench ... inspect the selected payment rail`
- changed the billing empty-state guidance to point users to the checkout workbench for the selected order
- changed the no-membership guidance in billing and recharge from `Settle a subscription order` to `Complete a subscription checkout`
- changed the recharge CTA from `Open billing queue` to `Open billing workbench`
- productized the billing lane and audit descriptions so they no longer describe user payment posture through `workspace settles`, `provider callback review`, or `operator-facing audit timeline`
- extended shared Portal i18n and `zh-CN` coverage for the updated payment-journey copy
- updated source-contract tests and relaxed the remaining regex to match the real multiline / trailing-comma `t(...)` call shape used by the billing page

## Runtime / Display Truth

### Billing Guidance Now Points To The Checkout Workbench

- the Portal now tells users to open the checkout workbench when they need to inspect or continue a pending payment path
- the payment journey copy no longer makes the raw queue session feel like the productized action surface

### Membership Activation Copy Now Matches Tenant Checkout Behavior

- the no-membership state now asks users to complete a subscription checkout instead of telling them to settle an order
- this keeps membership activation language aligned with the formal customer-facing payment journey already exposed in the workbench

### Billing And Recharge Surface Labels Are More Product-Facing

- the recharge page now routes users to a `billing workbench`, not a `billing queue`
- pending / failed / payment-history descriptions now speak in checkout, payment rail, and billing timeline terms rather than operator settlement or callback-review terminology

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by reducing the remaining bridge/operator wording in the Portal user payment journey
- improves Step 06 `8.6` evidence by keeping user-facing copy aligned with formal checkout posture and the checkout workbench boundary
- does not close Step 06 globally because explicit bridge/manual actions, callback rehearsal, and compatibility checkout-session fallback still remain in the runtime
- does not require `docs/é‹èˆµç€¯/*` writeback because this slice changes Portal presentation copy, guidance, and i18n coverage only; it does not change backend contracts, route authority, or architecture ownership
- `97` is satisfied through truthful no-writeback classification for this iteration

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## Remaining Follow-Up

1. Continue removing user-facing operator terminology from billing where the backend already exposes a formal tenant checkout path.
2. Decide whether the remaining provider-callback and manual-settlement wording inside explicit simulation surfaces should move behind an even clearer operator-only lane.
3. Continue Step 06 Portal commercialization closure until the entire billing journey reads as a tenant product surface end to end.
