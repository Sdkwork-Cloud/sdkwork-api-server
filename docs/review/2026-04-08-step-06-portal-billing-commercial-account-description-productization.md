# 2026-04-08 Step 06 Portal Billing Commercial Account Description Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by productizing the active tenant-facing commercial-account description that still exposed `canonical` and `state` implementation vocabulary in the Portal billing workbench, without changing billing contracts, commercial-account composition, or runtime behavior.

Execution boundary:

- keep the real commercial-account balance, hold, and identity flow unchanged
- keep Portal billing runtime behavior unchanged
- do not introduce new backend billing routes, SDK methods, or commerce payload changes

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `commercial-account-description-productization`
- Previous mode: `formal-checkout-attempt-description-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the visible commercial-account description so the tenant-facing Portal billing workbench stops saying `Commercial account exposes canonical balance, hold, and account identity state ...`.
   - `Priority Score: 181`
   - highest commercialization gain with a bounded display-only surface and no runtime risk

2. Leave the old `canonical balance` wording visible because the commercial-account facts are already correct.
   - `Priority Score: 25`
   - rejected because the tenant-facing Portal surface still leaked internal implementation naming even though the runtime facts were already correct

3. Broaden the slice into commercial-account runtime, repository, or backend contract refactors.
   - `Priority Score: 31`
   - rejected because the defect was wording only and widening the slice would add avoidable churn without solving a functional issue

### Chosen Action

Action 1 was selected because it removes a visible commercialization leak immediately while preserving the already-verified Portal billing commercial-account behavior.

## Root Cause Summary

### 1. The Commercial-Account Panel Still Exposed `Canonical` And `State` Vocabulary

The billing page still described the active commercial-account panel as `Commercial account exposes canonical balance, hold, and account identity state beside the workspace billing posture.` That phrase centers internal implementation naming rather than the tenant-facing product surface.

### 2. The Runtime Commercial-Account Facts Were Already Correct

The issue was not missing balance information, incorrect hold posture, or broken account identity state. The Portal billing workbench already loaded and displayed the relevant commercial-account facts correctly. The gap was strictly tenant-facing wording.

### 3. Shared I18n Coverage Needed To Stay In Lockstep

The new commercial-account description had to exist in both the billing page and the shared Portal i18n source contract. The `zh-CN` message catalog also needed a matching translation so the Portal surface would not fall back to English.

## Implemented Fixes

- replaced the commercial-account panel description with `Commercial account keeps balance, holds, and account identity visible beside the workspace billing posture.`
- removed the retired `Commercial account exposes canonical balance, hold, and account identity state ...` wording from the billing page source and shared Portal i18n contract
- updated shared Portal i18n and `zh-CN` translations so the new commercial-account description is fully registered
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
  - failed after the red-first test update because the shared Portal i18n source contract did not yet register the new commercial-account description
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because the billing page still rendered the older `Commercial account exposes canonical balance, hold, and account identity state ...` wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - failed after the red-first test update because the billing page source still contained the older `Commercial account exposes canonical balance, hold, and account identity state ...` description

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
  - full Portal Node suite returned `222 / 222` passing

## Current Assessment

### Closed In This Slice

- the Portal billing commercial-account panel no longer exposes `canonical` or `state` wording on the active tenant-facing surface
- shared Portal i18n and `zh-CN` now carry the new commercial-account description required by the active billing workbench
- the focused proof lanes now guard this commercial-account description contract against regression

### Still Open

- other billing wording outside the current commercial-account description cluster may still need future productization
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, or architecture ownership boundaries

## Next Slice Recommendation

1. Continue auditing whether tenant-facing billing surfaces still leak low-level finance implementation terminology.
2. Keep the next slice bounded to copy and proof unless a real backend or route contract blocker appears.
3. Continue Step 06 commercialization closure from the now more productized Portal billing baseline.
