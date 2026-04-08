# 2026-04-08 Step 06 Portal Billing Payment Reference Anchor Description Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by productizing the active tenant-facing payment-reference wording that still exposed `anchors` implementation language in the Portal billing checkout workbench, without changing billing contracts, checkout composition, or runtime behavior.

Execution boundary:

- keep the real checkout presentation and payment-reference facts unchanged
- keep Portal billing runtime behavior unchanged
- do not introduce new backend billing routes, SDK methods, or commerce payload changes

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `payment-reference-anchor-description-productization`
- Previous mode: `failed-payment-description-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the visible payment-reference detail so the tenant-facing Portal billing workbench stops saying `{reference} anchors the current {provider} / {channel} payment method for this order.`
   - `Priority Score: 186`
   - highest commercialization gain with a bounded display-only surface and no runtime risk

2. Leave the old `anchors` wording visible because the checkout reference behavior is already correct.
   - `Priority Score: 21`
   - rejected because the tenant-facing Portal surface still leaked implementation language even though the runtime facts were already correct

3. Broaden the slice into checkout reference runtime, repository, or backend contract refactors.
   - `Priority Score: 28`
   - rejected because the defect was wording only and widening the slice would add avoidable churn without solving a functional issue

### Chosen Action

Action 1 was selected because it removes a visible commercialization leak immediately while preserving the already-verified Portal billing checkout behavior.

## Root Cause Summary

### 1. The Checkout Detail Still Exposed `Anchors` Implementation Wording

The billing page still described the active checkout reference as `{reference} anchors the current {provider} / {channel} payment method for this order.` That phrase centers implementation semantics rather than tenant-facing product guidance.

### 2. The Runtime Checkout Facts Were Already Correct

The issue was not missing checkout references, broken provider/channel rendering, or incorrect checkout presentation selection. The Portal billing workbench already loaded and displayed the relevant payment facts correctly. The gap was strictly tenant-facing wording.

### 3. Shared I18n Coverage Needed To Stay In Lockstep

The new payment-reference description had to exist in both billing page render paths and the shared Portal i18n source contract. The `zh-CN` message catalog also needed a matching translation so the Portal surface would not fall back to English.

## Implemented Fixes

- replaced the checkout detail wording with `{reference} is the current {provider} / {channel} payment reference for this order.`
- removed the retired `{reference} anchors ...` wording from both billing page source paths and the shared Portal i18n contract
- updated shared Portal i18n and `zh-CN` translations so the new payment-reference description is fully registered
- tightened the Portal billing-i18n, product, and commercial-api proof lanes so the new wording is required and the retired wording is blocked

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
- `apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed after the red-first test update because the shared Portal i18n source contract did not yet register the new payment-reference description
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered the older `{reference} anchors ...` wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed after the red-first test update because the billing page source still contained the older `{reference} anchors ...` description

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `224 / 224` passing

## Current Assessment

### Closed In This Slice

- the Portal billing checkout detail no longer exposes `anchors` wording on the active tenant-facing surface
- shared Portal i18n and `zh-CN` now carry the new payment-reference description required by the active billing workbench
- the focused proof lanes now guard this payment-reference wording contract against regression

### Still Open

- other billing wording outside the current payment-reference description cluster may still need future productization
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, or architecture ownership boundaries

## Next Slice Recommendation

1. Continue auditing whether tenant-facing billing surfaces still leak low-level finance or implementation terminology.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized Portal billing baseline.
