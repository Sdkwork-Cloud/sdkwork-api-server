# 2026-04-08 Portal Billing Payment Attempt History Composition Step Update

## Slice Goal

Close the next Step 06 Portal payment-visibility gap by surfacing canonical order-scoped payment-attempt history inside the billing checkout workbench instead of limiting the user-visible panel to one `latest_payment_attempt`.

## Closed In This Slice

- hardened the billing repository so malformed payment-attempt payloads do not crash the workbench
- aligned the pending-order regression fixture to the real formal route:
  - `GET /portal/commerce/orders/{order_id}/payment-attempts`
- updated the Portal billing page to render canonical attempt history directly from `checkoutDetail.payment_attempts`
- added a visible `Latest attempt` marker in the checkout workbench
- added shared Portal i18n coverage and `zh-CN` translations for the new attempt-history copy

## Runtime / Display Truth

### Formal Attempt History Is Now Visible

- the billing workbench now reads canonical attempt history from `GET /portal/commerce/orders/{order_id}/payment-attempts`
- the `Checkout session` panel now shows:
  - attempt status
  - attempt sequence
  - latest-attempt marker
  - provider reference
  - initiated / updated timestamps
  - inline provider error detail when present

### Compatibility Bridge Still Remains For Panel Shell

- the existing checkout panel still uses compatibility checkout-session structure for the broader workbench shell
- operator settlement and callback simulation remain explicit bridge behavior
- formal payment-attempt history is now shown inside that bridge shell rather than waiting for a larger panel rewrite

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal exec tsc --noEmit`

## Remaining Follow-Up

1. Continue shrinking compatibility ownership in the checkout panel so canonical attempt and canonical method posture drive more of the state model.
2. Make retry versus reopen behavior more explicit for supported formal rails.
3. Keep non-canonical provider action paths out of the formal lane until the backend contract is truly ready.
