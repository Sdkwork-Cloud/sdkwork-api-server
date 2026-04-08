# 2026-04-08 Portal Billing Queue Action Workbench Boundary Closure Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by removing queue-level `settle` / `cancel` as the default pending-payment table posture and keeping those actions inside the opened checkout workbench, where explicit payment action semantics already belong.

## Closed In This Slice

- removed direct `Settle order` and `Cancel order` row actions from the default `Pending payment queue` table
- kept `settle_order` and `cancel_order` available only from the opened checkout workbench method cards by anchoring them to `activeCheckoutOrder`
- changed post-order queue status guidance so it now tells the user to open the checkout workbench to complete payment before quota or membership changes are applied
- extended shared Portal i18n and `zh-CN` coverage for the updated queue guidance copy
- updated source-contract tests so the new workbench-boundary behavior and queue guidance are locked in

## Runtime / Display Truth

### Queue Table No Longer Owns Settlement As A Default User Action

- the `Pending payment queue` now behaves as an order inventory and entry surface, not as the default place where settlement or cancellation is executed
- user-facing payment action semantics stay attached to the selected checkout session instead of being duplicated at the queue row level

### Checkout Workbench Now Owns Remaining Explicit Bridge Actions

- once a user opens the checkout workbench, `settle_order` and `cancel_order` remain available there as explicit method-card actions
- this keeps the remaining bridge/manual actions inside the payment workbench context where selected order, rail, and checkout state are already visible

### Post-Order Guidance Now Points To The Correct Surface

- after order creation, the billing page now tells the user that the order was queued and that payment should be completed from the checkout workbench
- this avoids implying that queue-level settlement is the normal customer-facing next step

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by tightening the action boundary between the queue inventory surface and the payment workbench surface
- improves Step 06 `8.6` evidence by keeping explicit bridge/manual actions in the already-opened checkout workbench instead of leaving them on the default queue grid
- does not close Step 06 globally because payment simulation posture, compatibility checkout-session fallback, and explicit bridge actions still exist inside the workbench
- does not require `docs/é‹èˆµç€¯/*` writeback because this slice changes Portal interaction boundaries and copy only; it does not change backend contracts, route authority, or architecture ownership
- `97` is satisfied through truthful no-writeback classification for this iteration

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## Remaining Follow-Up

1. Continue separating compatibility bridge actions from default customer-facing Portal payment surfaces wherever the backend already provides formal payment truth.
2. Decide whether the remaining workbench-hosted `settle_order` and `cancel_order` actions should stay in the billing product surface long term or move behind a clearer operator-only boundary.
3. Continue Step 06 Portal commercialization closure until the pending-payment journey is formal-first across queue inventory, workbench actions, and release evidence.
