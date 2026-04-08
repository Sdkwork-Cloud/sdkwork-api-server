# 2026-04-08 Portal Billing Checkout Retry / Reopen Decision Clarity Step Update

## Slice Goal

Close the next Step 06 Portal billing workbench ambiguity by making provider checkout launch behavior explicit and canonical-attempt-driven instead of leaving resume-versus-retry behavior implicit inside the page.

## Closed In This Slice

- added `buildBillingCheckoutLaunchDecision(...)` in Portal billing services
- formalized three provider checkout launch outcomes:
  - `resume_existing_attempt`
  - `create_retry_attempt`
  - `create_first_attempt`
- updated the Portal billing page to:
  - compute provider checkout launch posture from canonical `payment_attempts`
  - show distinct in-flight status copy for reopen versus fresh retry
  - label the provider CTA as:
    - `Resume provider checkout`
    - `Retry with new attempt`
    - `Launch provider checkout`
  - explain the chosen launch posture inline on the checkout method card
- extended shared Portal i18n and `zh-CN` coverage for the new decision copy

## Runtime / Display Truth

### Canonical Attempt Posture Now Drives Checkout Launch Intent

- if the latest canonical attempt still has a reusable checkout URL, the workbench reopens that session
- if a canonical attempt exists but is no longer reusable, the workbench creates a fresh retry attempt
- if no canonical attempt exists for the method yet, the workbench creates the first attempt

### Compatibility Bridge Still Remains

- compatibility checkout-session still contributes panel-shell context
- operator settlement and callback rehearsal remain explicit bridge behavior
- this slice does not remove the larger compatibility checkout-session container

## Architecture / Acceptance Impact

- advances Step 06 Portal commercialization closure by making the attempt-backed provider launch rule explicit and testable
- improves `8.6` evidence for the Portal billing workbench, but does not close Step 06 globally
- does not require `docs/架构/133-*` writeback because no control-plane boundary, backend contract, or architecture truth changed in this slice
- `97` remains satisfied through truthful no-writeback classification for architecture docs in this iteration

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## Remaining Follow-Up

1. Continue reducing compatibility checkout-session dependence in the pending-payment workbench without inventing frontend-only payment semantics.
2. Keep retry / reopen behavior anchored to canonical payment-attempt truth if more provider rails are introduced.
3. Continue the broader Step 06 payment closure until the Portal checkout workbench is fully attempt-backed and commercial evidence-complete.
