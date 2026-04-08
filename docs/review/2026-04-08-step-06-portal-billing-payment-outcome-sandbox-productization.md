# 2026-04-08 Step 06 Portal Billing Payment Outcome Sandbox Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by replacing the remaining tenant-facing provider-event replay wording inside the billing sandbox with payment-outcome language, without changing payment runtime contracts, sandbox behavior, or backend ownership boundaries.

Execution boundary:

- keep the existing `sendBillingPaymentEvent` runtime path intact
- keep sandbox `event_type`, provider targeting, replay id generation, and checkout-method binding unchanged
- do not change billing repository contracts, service decisions, or provider callback processing behavior
- do not introduce new backend payment routes, new billing panels, or new runtime state

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `payment-outcome-sandbox-productization`
- Previous mode: `provider-handoff-vocabulary-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the remaining provider-event replay copy inside the billing sandbox so the visible surface speaks in payment-outcome language.
   - `Priority Score: 176`
   - highest commercialization gain with a bounded copy-only surface and no runtime risk

2. Leave the provider-event replay wording in place because the sandbox is inherently technical.
   - `Priority Score: 47`
   - rejected because Step 06 still requires the tenant-facing workbench to read like a product surface even where sandbox posture remains visible

3. Rename runtime provider-event concepts and transport fields to match the new product copy.
   - `Priority Score: 52`
   - rejected because it would widen the slice into contract churn without solving a runtime bug

### Chosen Action

Action 1 was selected because it closes a visible commercialization leak immediately while preserving the already-verified sandbox behavior, checkout-method targeting, and backend event semantics.

## Root Cause Summary

### 1. The Sandbox Still Exposed Provider-Event Replay Mechanics

The billing workbench had already been partially productized, but the following tenant-facing sandbox surfaces still used provider-event wording directly:

- the capability badge `Provider events`
- the sandbox guidance sentence about replaying provider settlement, failure, or cancellation events
- the in-progress status messages beginning with `Replaying ...`
- the action buttons `Replay settlement event`, `Replay failure event`, and `Replay cancel event`

That wording described the surface as an integration replay console instead of a product-facing billing sandbox.

### 2. The Runtime Facts Were Already Correct

The issue was not missing sandbox state or broken replay behavior. The billing page already targeted the correct checkout method, emitted the correct `event_type`, and refreshed the same billing outcomes from the canonical backend response. The gap was strictly presentation-language quality on the tenant-facing workbench.

### 3. One Proof Lane Still Lagged Behind The Product Copy

After the page and shared i18n changes, focused proof passed, but the first full Portal suite surfaced a stale assertion in `portal-payment-rails.test.mjs` that still required `Provider events`. This slice therefore needed to:

- replace provider-event replay wording in the billing page source
- replace the same wording in shared i18n and `zh-CN`
- align the focused billing proof lanes
- align the broader payment-rails proof lane so it validates current product truth instead of retired vocabulary

## Implemented Fixes

- updated the Portal billing sandbox badge from `Provider events` to `Payment outcomes`
- updated the sandbox guidance sentence to:
  - `Apply settlement, failure, or cancellation outcomes for the selected payment method before live payment confirmation is enabled.`
- updated in-progress sandbox status copy to:
  - `Applying {provider} settlement outcome for {orderId}...`
  - `Applying {provider} failure outcome for {orderId}...`
  - `Applying {provider} cancellation outcome for {orderId}...`
- updated sandbox action labels to:
  - `Apply settlement outcome`
  - `Apply failure outcome`
  - `Apply cancellation outcome`
- updated the shared Portal i18n registry and `zh-CN` messages so the new payment-outcome vocabulary is registered consistently
- updated `portal-payment-rails.test.mjs` so the broader proof lane now matches the current product wording

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
- `apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed after the red-first test update because the shared i18n registry still exposed provider-event replay wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered `Provider events`, `Replay ... event`, and `Replaying ...` copy

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- first full Portal suite run:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - returned `220 / 221` passing because `portal-payment-rails.test.mjs` still asserted the retired `Provider events` label
- proof-lane recovery:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- final full-suite verification:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- the billing sandbox no longer exposes `Provider events`, `Replay ... event`, or `Replaying ...` wording on the tenant-facing surface
- shared Portal i18n and `zh-CN` now describe the same sandbox capability through payment-outcome language
- the billing-focused proof lanes plus the broader payment-rails proof lane now guard against regression back to provider-event replay wording

### Still Open

- `Payment event sandbox` and `Event target` still remain as explicit sandbox vocabulary
- `Event signature` still remains visible as a low-level detail label inside checkout method evidence
- manual settlement bridge behavior still remains when simulation posture permits it
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, component ownership, or architecture contracts

## Next Slice Recommendation

1. Continue auditing whether `Payment event sandbox`, `Event target`, and `Event signature` should remain explicit sandbox terms or be further productized.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized payment-outcome sandbox baseline.
