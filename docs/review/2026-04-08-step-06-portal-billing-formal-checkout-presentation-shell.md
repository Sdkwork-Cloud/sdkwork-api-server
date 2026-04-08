# 2026-04-08 Step 06 Portal Billing Formal Checkout Presentation Shell Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by reducing the remaining compatibility checkout-session ownership inside the pending-payment workbench shell.

Execution boundary:

- keep the existing Portal `Checkout session` workbench intact
- keep formal order/payment-method/payment-attempt truth primary
- keep compatibility checkout-session only as the bridge for operator settlement, callback rehearsal, and residual fallback facts
- do not widen the UI into a new attempt-first payment console

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `formal-checkout-presentation-shell`
- Previous mode: `checkout-retry-reopen-decision-clarity`
- Strategy switch: no

### Candidate Actions

1. Add a reusable formal-first checkout presentation helper and route the billing shell through it.
   - `Priority Score: 167`
   - highest commercial closure value for the smallest stable write surface

2. Rebuild the pending-payment panel into a brand-new attempt-first payment console.
   - `Priority Score: 76`
   - rejected because it widens Step 06 scope and introduces new UI semantics not yet backed by the runtime

3. Keep the current compatibility-first shell and wait for a later larger rewrite.
   - `Priority Score: 29`
   - rejected because the main billing workbench would continue to present stale compatibility summary facts even after canonical truth was already available

### Chosen Action

Action 1 was selected because the backend and repository layers already expose enough canonical payment truth to formalize shell presentation now, without fabricating new actions or destabilizing the current workbench.

## Root Cause Summary

### 1. The Billing Shell Still Presented Compatibility Reference And Rail Identity

The previous Step 06 slices had already introduced:

- formal order detail
- formal payment-method detail
- formal payment-attempt launch and history

But the visible pending-payment shell still preferred:

- `checkout_session.reference`
- `checkout_session.session_status`
- `checkout_session.guidance`
- `checkout_session.payable_price_label`

That kept a compatibility aggregate in charge of the most operator-visible commercial facts.

### 2. Canonical Payment Truth Was Not Yet Composed Into A Named Presentation Model

The page had canonical facts available, but there was no reusable service boundary that answered:

- which reference is authoritative now
- which provider / channel currently anchors the order
- whether current status comes from a payment attempt or compatibility session
- whether guidance should come from an attempt error, launch decision, or compatibility fallback

Without that boundary, the page kept ad-hoc shell composition and remained regression-prone.

### 3. Shared Portal I18n Did Not Yet Cover The Formal-First Shell Copy

The shell transition required shared copy for:

- `Primary rail`
- `Current status`
- formal reference anchor text
- empty formal guidance fallback

Without shared i18n coverage, the new shell posture would either remain implicit or leak page-local copy.

## Implemented Fixes

- added `buildBillingCheckoutPresentation(...)` to `sdkwork-router-portal-billing` services
- the new helper now derives, in formal-first order:
  - reference from canonical latest payment attempt, then canonical checkout method, then compatibility session fallback
  - provider / channel from canonical checkout method, then compatibility session provider fallback
  - status from canonical payment attempt, then compatibility session fallback
  - guidance from canonical payment-attempt error, then formal launch decision, then compatibility session fallback
- updated the Portal billing page to:
  - compute checkout loading/status text from the formal presentation helper
  - replace `Checkout mode` and `Session status` shell facts with `Primary rail` and `Current status`
  - show formal-first guidance text instead of trusting compatibility `checkoutSession.guidance`
  - show formal-first reference and payable price in the payment-rail summary panel
  - retain explicit fallback references to `checkoutDetail?.selected_payment_method` so the broader product proof suite remains aligned
- updated shared Portal i18n registries and `zh-CN` translations for the new formal shell copy

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/services/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
- `apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed before the fix because `buildBillingCheckoutPresentation(...)` did not exist and the billing page still rendered compatibility-first shell facts
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed before the fix because the billing page and shared Portal i18n registry still lacked the new formal shell copy

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## Current Assessment

### Closed In This Slice

- the billing shell no longer depends on compatibility checkout-session as the primary source for visible reference, status, and guidance posture
- canonical payment truth now drives both the loading/status narration and the persistent workbench shell facts
- the new shell posture is reusable, named, and regression-safe at the service level
- shared Portal i18n now owns the new commercial shell copy

### Still Open

- operator settlement and provider callback rehearsal remain compatibility bridge behavior
- the larger checkout workbench still depends on compatibility checkout-session for some interactive shell and simulation posture
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed presentation composition inside an existing formal payment lane, but it did not change route authority, backend API truth, control-plane ownership, or architecture boundary contracts

## Next Slice Recommendation

1. Continue removing compatibility ownership from the checkout workbench only where formal backend truth already exists.
2. Keep operator-only bridge behavior clearly separated from tenant-facing formal payment truth.
3. Continue Step 06 commercialization closure until release evidence can show the pending-payment workbench is truth-aligned end to end.
