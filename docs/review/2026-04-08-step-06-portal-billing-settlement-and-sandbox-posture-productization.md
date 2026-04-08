# 2026-04-08 Step 06 Portal Billing Settlement And Sandbox Posture Productization Review

## Scope

This slice continued the Step 06 Portal commercialization closure lane by cleaning up the remaining settlement-summary and explicit simulation wording in billing, without changing payment runtime contracts, action availability, or backend ownership boundaries.

Execution boundary:

- keep the existing billing summary facts and explicit payment simulation behavior intact
- keep formal order, payment-method, and payment-event truth primary
- do not remove the sandbox itself in this slice
- do not introduce new backend payment routes, new operator consoles, or new simulation behavior

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `settlement-and-sandbox-posture-productization`
- Previous mode: `payment-journey-copy-productization`
- Strategy switch: no

### Candidate Actions

1. Productize the remaining `Commercial settlement rail` and `Provider callbacks` surfaces so they read as billing coverage and sandbox tooling instead of operator or callback-rehearsal posture.
   - `Priority Score: 174`
   - highest commercialization gain with the smallest change surface and no runtime risk

2. Remove the payment-event sandbox from Portal billing entirely.
   - `Priority Score: 96`
   - rejected because it widens the slice into structural product behavior and operator-boundary changes not yet backed by runtime or route changes

3. Leave the existing wording in place because the sandbox is already behind a simulation flag.
   - `Priority Score: 41`
   - rejected because Step 06 still requires the visible tenant-facing workbench language to read like a commercial product, not an internal rehearsal tool

### Chosen Action

Action 1 was selected because it closes a visible commercialization gap immediately while preserving the already-verified payment behavior and simulation gating.

## Root Cause Summary

### 1. The Billing Summary Still Framed Commercial Facts Through Operator Language

The billing workspace already showed useful commercial facts:

- benefit lots
- credit holds
- request settlement capture

but the panel title and description still called that area `Commercial settlement rail` and an `operator-facing posture`. That wording made the default tenant billing summary read like an internal control-plane panel.

### 2. The Simulation Panel Still Read Like Infrastructure Rehearsal Copy

The explicit simulation panel still used wording such as:

- `Provider callbacks`
- `Callback rail`
- `callback rehearsal`
- `Simulate provider settlement`

That was technically recognizable, but it described the surface as a callback tool instead of a clearly bounded payment-event sandbox inside the Portal product.

### 3. Shared I18n Truth Needed To Move In Lockstep

Because the Portal i18n layer uses source-contract tests and a shared message registry, the copy change required synchronized updates to:

- the billing page source
- the shared message-key registry
- `zh-CN` translations
- product/workspace/i18n proof lanes

## Implemented Fixes

- updated the Portal billing page to:
  - rename `Commercial settlement rail` to `Settlement coverage`
  - replace the operator-facing settlement description with billing-snapshot copy
  - rename `Provider callbacks` to `Payment event sandbox`
  - relabel the sandbox badge, selector, placeholder, active-rail sentence, and action buttons to sandbox payment-event wording
- updated shared Portal i18n registries and `zh-CN` translations for the new settlement/sandbox copy
- updated source-contract tests so they now:
  - require `Settlement coverage`
  - require `Payment event sandbox`
  - reject the retired settlement-rail / provider-callback / callback-rehearsal wording

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
- `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Red First

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - failed after the red-first test update because billing still rendered `Commercial settlement rail` and `Provider callbacks`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - failed after the red-first test update because the shared Portal i18n registry still exposed the retired callback and simulation wording
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
  - failed after the red-first test update because the commercial workspace test still saw the old settlement panel title in billing

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## Current Assessment

### Closed In This Slice

- the billing summary no longer markets commercial facts through an operator-facing settlement title
- the explicit simulation surface now reads as a bounded `Payment event sandbox`
- shared Portal i18n and `zh-CN` now cover the new settlement/sandbox product copy
- the product and workspace proof lanes now guard against regression back to the retired callback-rehearsal wording

### Still Open

- explicit payment-event sandbox behavior still remains in the billing runtime
- manual settlement bridge behavior still remains when simulation posture permits it
- compatibility checkout/session bridge posture still remains in the broader Step 06 lane
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration changed only Portal presentation copy, i18n coverage, and proof contracts; it did not change backend routes, runtime authority, component ownership, or architecture contracts

## Next Slice Recommendation

1. Continue auditing billing for any remaining copy that still exposes operator or bridge concepts as the default tenant posture.
2. Evaluate whether the explicit payment-event sandbox should eventually move behind a narrower operator-only boundary after the product surface is fully clean.
3. Continue Step 06 commercialization closure with the next smallest Portal billing surface that still exposes internal payment vocabulary.
