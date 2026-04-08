# 2026-04-08 Portal Billing Formal Checkout Presentation Shell Step Update

## Slice Goal

Close the next Step 06 Portal billing shell gap by moving pending-payment reference, rail identity, status, and guidance presentation onto canonical order/payment-method/payment-attempt truth before compatibility checkout-session fallback.

## Closed In This Slice

- added `buildBillingCheckoutPresentation(...)` in Portal billing services
- formalized a reusable presentation model for:
  - checkout reference
  - payable price label
  - payment method name
  - provider / channel
  - current status source
  - current guidance source
  - provider-launch decision posture when a formal handoff rail exists
- updated the Portal billing page to:
  - build loading/status copy from the formal presentation model
  - replace compatibility-first shell facts with:
    - `Primary rail`
    - `Current status`
    - formal-first guidance text
    - formal-first selected reference
    - formal-first payable price
  - keep explicit fallback references to `checkoutDetail?.selected_payment_method` so the broader product proof lane remains aligned
- extended shared Portal i18n and `zh-CN` coverage for the new formal shell copy

## Runtime / Display Truth

### Formal Presentation Now Owns The Main Pending-Payment Shell

- checkout reference now prefers canonical latest payment-attempt reference
- provider and channel now prefer canonical checkout-method posture
- status now prefers canonical payment-attempt status and falls back to checkout-session status only when attempt truth is absent
- guidance now prefers canonical payment-attempt error or formal provider-launch decision detail before compatibility guidance fallback

### Compatibility Checkout Session Is Reduced To Bridge / Fallback Ownership

- operator settlement and provider callback rehearsal still remain compatibility bridge behavior
- compatibility checkout-session still remains the interactive bridge for payment simulation posture and residual fallback facts
- this slice does not invent a new attempt-scoped backend console or new provider semantics

## Architecture / Acceptance Impact

- advances Step 06 commercialization closure by turning the billing shell into a formal-first presentation surface instead of a compatibility-first summary
- improves Step 06 `8.6` evidence for Portal payment truth presentation, but does not close Step 06 globally
- does not require `docs/架构/*` writeback because this slice does not change control-plane ownership, backend contracts, route authority, or architecture boundaries
- `97` is satisfied through truthful no-writeback classification for architecture docs in this iteration

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## Remaining Follow-Up

1. Continue shrinking compatibility checkout-session ownership in the pending-payment workbench without widening scope into a new payment console.
2. Keep provider callback rehearsal and operator settlement explicitly classified as bridge behavior until formal backend semantics replace them.
3. Continue Step 06 Portal commercialization closure until the checkout workbench is fully truth-aligned across shell, actions, verification, and release evidence.
