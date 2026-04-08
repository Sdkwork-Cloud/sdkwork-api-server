# 2026-04-08 Step 06 Portal Billing Callback Confirmation Vocabulary Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by replacing the remaining tenant-facing `callback flow` and `provider callback` wording in billing replay outcome feedback with payment-confirmation language, without changing payment runtime contracts, replay behavior, or backend ownership boundaries.

Execution boundary:

- keep the existing replay, settlement, failure, and cancellation runtime paths intact
- keep provider event truth and billing state transitions unchanged
- do not change checkout launch, payment-method composition, sandbox posture, or callback processing behavior
- do not introduce new backend payment routes, new billing panels, or new runtime state

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `callback-confirmation-vocabulary-productization`
- Previous mode: `payment-method-vocabulary-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the remaining replay outcome wording so Portal billing describes state changes as happening after provider payment confirmation instead of callback flow mechanics.
   - `Priority Score: 173`
   - highest commercialization gain with a bounded copy-only surface and no runtime risk

2. Leave the callback wording in place because it is technically accurate to the provider integration.
   - `Priority Score: 42`
   - rejected because Step 06 still requires tenant-facing billing feedback to read like a product surface, not an integration console

3. Broaden the slice into replay runtime renames and backend terminology cleanup.
   - `Priority Score: 57`
   - rejected because it would widen the iteration beyond the proven product-copy gap and create unnecessary contract risk

### Chosen Action

Action 1 was selected because it closes a visible commercialization leak immediately while preserving the already-verified replay behavior, billing workbench shape, and backend callback semantics.

## Root Cause Summary

### 1. Replay Outcome Copy Still Exposed Integration Callback Mechanics

The billing page had already been partially productized, but replay-result feedback still surfaced callback terminology directly to tenants:

- `{targetName} was settled through the {provider} callback flow.`
- `{targetName} was marked failed after the {provider} callback.`
- `{targetName} was canceled through the {provider} callback flow.`

That wording described the experience as an internal provider callback pipeline instead of a tenant-facing billing confirmation flow.

### 2. The Runtime Facts Were Already Correct

The issue was not missing state or broken replay behavior. The billing page already rendered the correct settlement, failure, and cancellation outcomes from the canonical replay result. The gap was strictly presentation-language quality on the tenant-facing workbench.

### 3. Shared I18n Truth Needed To Move With The Page Copy

The replay outcome strings were shared across the billing page, common i18n registry, and `zh-CN` messages. This slice needed to:

- replace callback wording in the billing page source
- replace callback wording in shared i18n fallback strings
- align `zh-CN` translations with the new payment-confirmation language
- keep source-contract tests strict enough to reject regression back to callback-flow wording

## Implemented Fixes

- updated the Portal billing page replay-result wording to:
  - `{targetName} was settled after the {provider} payment confirmation.`
  - `{targetName} was marked failed after the {provider} payment confirmation.`
  - `{targetName} was canceled after the {provider} payment confirmation.`
- updated the shared Portal i18n registry so provider-generic fallback strings now say:
  - `{targetName} was settled after provider payment confirmation.`
  - `{targetName} was canceled after provider payment confirmation.`
- updated `zh-CN` coverage so the new payment-confirmation vocabulary is localized consistently
- tightened billing i18n and product proof so the retired `callback flow` / `provider callback` terminology now fails source-contract verification

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
- `apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed after the red-first test update because the shared i18n registry still exposed `callback flow` / `provider callback` wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered callback-flow wording in replay outcome copy

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- the billing workbench no longer exposes `callback flow` or `provider callback` terminology in replay outcome feedback
- shared Portal i18n and `zh-CN` now describe replay results through payment-confirmation language
- the billing i18n and product proof lanes now guard against regression back to callback-flow wording

### Still Open

- `Provider handoff` still remains in billing copy and method labeling
- replay action labels still describe `Replaying {provider} settlement/failure/cancellation...`, which may still read as provider tooling rather than product-facing confirmation operations
- manual settlement bridge behavior still remains when simulation posture permits it
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, component ownership, or architecture contracts

## Next Slice Recommendation

1. Continue auditing Portal billing for remaining provider-bridge phrases, starting with `Provider handoff`.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized replay-feedback baseline.
