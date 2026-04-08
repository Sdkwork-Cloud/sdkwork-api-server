# 2026-04-08 Step 06 Portal Billing Fallback Reason Description Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by productizing the fallback-reason description that still exposed an operator-facing viewpoint on the tenant-facing Portal billing workbench, without changing billing analytics contracts, routing ownership boundaries, or runtime behavior.

Execution boundary:

- keep billing event analytics cards, fallback evidence rows, and routing counters unchanged
- keep Portal billing runtime behavior unchanged
- do not introduce new backend billing routes, routing models, or analytics computations

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `fallback-reason-description-productization`
- Previous mode: `payment-history-refund-status-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the visible fallback-reason description so Portal billing explains degraded routing from the user viewpoint rather than the operator viewpoint.
   - `Priority Score: 181`
   - highest commercialization gain with a bounded display-only surface and no runtime risk

2. Leave the old wording visible because the routing fallback evidence is already correct.
   - `Priority Score: 26`
   - rejected because the remaining `operators can distinguish` phrasing still leaks internal operational language into the tenant-facing billing surface

3. Broaden the slice into routing analytics contracts or billing repository changes.
   - `Priority Score: 35`
   - rejected because the issue was wording only and widening the slice would add runtime churn without solving a functional defect

### Chosen Action

Action 1 was selected because it removes a visible commercialization leak immediately while preserving the already-verified fallback evidence rendering and Portal billing runtime behavior.

## Root Cause Summary

### 1. The Billing Fallback Description Still Spoke In Operator Terms

The billing page still described fallback reasoning as something `operators can distinguish` from `normal preference selection`. That wording centers internal operational review rather than the tenant-facing billing experience.

### 2. The Runtime Facts Were Already Correct

The issue was not missing fallback evidence, broken routing analytics, or incorrect billing counters. The Portal billing workbench already loaded and displayed the relevant fallback facts correctly. The gap was strictly tenant-facing wording.

### 3. Shared I18n Coverage Needed To Match The Replacement

The new fallback-reason description needed to exist in both the billing page and the shared Portal i18n source contract. The `zh-CN` message catalog also needed a matching translation so the Portal surface would not fall back to English.

## Implemented Fixes

- replaced the fallback-reason description with the new product-facing explanation centered on degraded routing versus the preferred routing path
- removed the retired operator-facing wording from the billing page source and shared Portal i18n contract
- updated shared Portal i18n and `zh-CN` translations so the new fallback-reason description is fully registered
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
  - failed after the red-first test update because the shared Portal i18n source contract did not yet register the new fallback-reason description
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered the operator-facing fallback wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed after the red-first test update because the billing page source still contained `operators can distinguish degraded routing from normal preference selection`

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `221 / 221` passing

## Current Assessment

### Closed In This Slice

- the Portal billing fallback-reason description no longer exposes operator-facing wording as tenant-facing copy
- shared Portal i18n and `zh-CN` now carry the new fallback-reason description required by the active billing workbench
- the focused proof lanes now guard this fallback-reason description contract against regression

### Still Open

- other low-level billing wording outside the current fallback-reason description cluster may still need future productization
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/é‹èˆµç€¯/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, or architecture ownership boundaries

## Next Slice Recommendation

1. Continue auditing whether tenant-facing billing surfaces still leak low-level finance or routing review terminology.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized Portal billing finance baseline.
