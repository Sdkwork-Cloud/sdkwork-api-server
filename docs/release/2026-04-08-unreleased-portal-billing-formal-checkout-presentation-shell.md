# 2026-04-08 Unreleased Portal Billing Formal Checkout Presentation Shell

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `formal-checkout-presentation-shell`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Add a reusable formal-first checkout presentation helper and route the pending-payment shell through it.
2. Rebuild the billing workbench into a new attempt-first payment console.
3. Leave the current compatibility-first shell in place until a later rewrite.

Action `1` was selected because it closes a user-visible commercial truth gap immediately, without inventing new backend semantics or destabilizing the existing workbench.

## 3. Actual Changes

- added `buildBillingCheckoutPresentation(...)` to `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/services/index.ts`
  - composes canonical reference, provider, channel, status, guidance, and launch-decision posture before compatibility fallback
- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - pending-payment loading/status copy now comes from the formal presentation model
  - shell facts now render formal-first `Primary rail`, `Current status`, guidance, reference, and payable price
  - compatibility checkout-session remains only as bridge / fallback input for the shell
- updated shared Portal i18n registries and `zh-CN` translations for the new formal shell copy
- kept explicit fallback references to `checkoutDetail?.selected_payment_method` so the wider product proof suite remains aligned while formal-first presentation stays primary

## 4. Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## 5. Architecture / Delivery Impact

- the Portal billing shell now presents operator-facing payment facts from canonical truth instead of compatibility-first summary state
- formal payment-attempt and checkout-method posture now visibly reach the persistent pending-payment workbench shell
- this slice improves Step 06 commercial evidence without overstating whole-step completion

## 6. Risks / Limits

- compatibility checkout-session is still retained for bridge behavior and fallback composition
- operator settlement and callback rehearsal are still not replaced by fully formal runtime semantics
- this slice does not add new provider rails, new backend payment actions, or a new payment console

## 7. Next Entry

1. Continue reducing compatibility ownership in the pending-payment workbench where formal truth already exists.
2. Keep future checkout interaction changes tied to real backend semantics only.
3. Continue the Step 06 commercialization lane from the now formal-first billing shell surface.
