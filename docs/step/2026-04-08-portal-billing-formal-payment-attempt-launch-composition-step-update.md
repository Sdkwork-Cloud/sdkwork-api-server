# 2026-04-08 Portal Billing Formal Payment Attempt Launch Composition Step Update

## Slice Goal

Close the next Step 06 Portal payment-action gap by wiring the billing checkout workbench onto the formal payment-attempt launch contract instead of leaving provider checkout launch as a compatibility-only bridge.

## Closed In This Slice

- added Portal TS support for formal payment-attempt creation input
- added Portal API client methods for order-scoped payment-attempt listing and creation
- added billing repository support for formal payment-attempt creation
- updated the Portal billing page so eligible provider rails now:
  - open the latest canonical checkout URL when one already exists
  - create a fresh canonical payment attempt when a new provider launch is needed
  - derive `success_url` and `cancel_url` from the current Portal route
- added shared Portal i18n coverage for the new launch status and button copy

## Runtime / Display Truth

### Formal-First Launch Path

- the Portal billing page now uses `POST /portal/commerce/orders/{order_id}/payment-attempts` for the supported formal launch lane
- the visible billing workbench now treats canonical `checkout_url` from the latest payment attempt as the first reopening path

### Compatibility Bridge Still In Use

- compatibility checkout-session detail still anchors the existing `Checkout session` panel shape
- operator settlement and callback simulation still remain compatibility bridge actions
- non-Stripe provider handoff rails still do not use formal launch because the backend create-attempt path is not yet canonical for them

## Provider Scope In This Slice

Formal launch is intentionally limited to:

- `provider === 'stripe'`
- `channel === 'hosted_checkout'`
- `action === 'provider_handoff'`
- `availability === 'available'`

This keeps frontend behavior aligned with the real backend implementation instead of pretending all rails are already formalized.

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal exec tsc --noEmit`

## Remaining Follow-Up

1. Surface attempt history and retry posture more explicitly inside the billing workbench.
2. Continue reducing order-scoped compatibility actions to a narrow operator-only bridge.
3. Expand formal launch only when additional provider rails have real canonical backend support.
