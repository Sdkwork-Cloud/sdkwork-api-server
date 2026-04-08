# 2026-04-08 Portal Billing Operator Bridge Visibility Closure Step Update

## Slice Goal

Close the next Step 06 Portal billing commercialization gap by removing operator-bridge posture from the default pending-payment rail summary and normal checkout-method presentation, while keeping lab-only settlement and callback rehearsal behind the existing payment-simulation gate.

## Closed In This Slice

- filtered the pending-payment checkout workbench method list so `settle_order` no longer appears in the default user-facing checkout method grid when payment simulation is off
- changed the remaining operator-settlement page copy to the more truthful product label `Manual settlement`
- rebuilt the secondary `Payment rail` summary panel so it now stays formal-first and only summarizes:
  - primary rail
  - current selected reference
  - payable price
- removed the hardcoded `Local desktop mode` / `Operator settlement` and `Server mode handoff` shell rows that previously made the compatibility bridge look like canonical payment truth
- extended shared Portal i18n and `zh-CN` coverage for the new formal rail summary copy

## Runtime / Display Truth

### Formal Checkout Now Owns The Payment Rail Summary

- the `Payment rail` workspace panel now describes formal checkout posture instead of mixing operator bridge and provider handoff into the same default summary
- the main rail summary now prefers canonical provider/channel presentation and avoids falling back to compatibility `manual_lab` provider posture unless simulation mode is intentionally active

### Operator Bridge Visibility Is Reduced To Explicit Lab Posture

- `Manual settlement` remains an operator/lab concept instead of the default user-facing rail label
- provider callback rehearsal remains gated by `payment_simulation_enabled`
- queue-level `Settle order` and callback simulation actions still exist only inside the intentional payment-simulation posture

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by reducing the remaining operator/bridge language inside the Portal user payment workbench
- improves Step 06 `8.6` evidence for separating formal tenant payment truth from lab-only operator bridge posture
- does not close Step 06 globally because compatibility checkout-session, queue settlement, and callback rehearsal still remain available in the simulation lane
- does not require `docs/架构/*` writeback because this slice only changes Portal presentation visibility and copy classification, not backend contracts, route authority, or architecture boundaries
- `97` is satisfied through truthful no-writeback classification for this iteration

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## Remaining Follow-Up

1. Continue separating compatibility bridge posture from the main checkout workbench wherever the backend already exposes formal payment truth.
2. Decide whether the remaining queue-level settlement and callback rehearsal actions should stay in billing long term or move behind a clearer operator-only lane.
3. Continue Step 06 Portal commercialization closure until the pending-payment workbench is formal-first across shell, actions, verification, and release evidence.
