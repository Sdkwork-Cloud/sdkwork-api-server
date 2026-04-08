# 2026-04-08 Step 06 Portal Billing Checkout Retry / Reopen Decision Clarity

## Scope

This slice continued the Step 06 Portal commercialization closure lane by making the billing checkout workbench explain whether it will reopen an existing provider checkout or create a fresh payment attempt.

Execution boundary:

- keep the existing Portal `Checkout session` workbench intact
- keep canonical payment-attempt truth as the decision source
- do not invent new backend payment semantics
- do not widen the UI into a new standalone payment console

## Decision Ledger

- Date: `2026-04-08`
- Version: `Unreleased`
- Wave / Step: `B / 06`
- Primary mode: `checkout-retry-reopen-decision-clarity`
- Previous mode: `payment-attempt-history-composition`
- Strategy switch: no

### Candidate Actions

1. Formalize provider checkout launch posture into a reusable service decision helper and expose the decision explicitly in the Portal UI.
   - `Priority Score: 161`
   - closes a user-visible commercial ambiguity with the smallest stable write surface

2. Always create a fresh payment attempt whenever a user clicks the provider checkout CTA.
   - `Priority Score: 69`
   - rejected because it discards valid canonical checkout reuse and increases provider retry noise

3. Leave the current inline page logic in place and keep resume-versus-retry behavior implicit.
   - `Priority Score: 28`
   - rejected because operators and tenants cannot tell whether the workbench is reopening an existing checkout or creating a new attempt

### Chosen Action

Action 1 was selected because the backend already exposes canonical order-scoped payment attempts, and the Portal should turn that truth into an explicit, testable launch decision instead of hiding it in page-local branching.

## Root Cause Summary

### 1. Provider Checkout Launch Semantics Were Implicit

The billing page could already decide to reopen an existing provider checkout or create a new attempt, but that rule lived inline inside the page and was not formalized as a named behavior.

That created a real product gap:

- users could not tell whether clicking the CTA would reopen or retry
- reviewers could not test the decision independently from the page
- future slices risked re-implementing the same rule inconsistently

### 2. The UI Did Not Explain Why A Different CTA Was Shown

The pending-payment workbench surfaced provider handoff actions, but it did not clearly explain:

- when the latest attempt was still reusable
- when a failed or expired attempt forced a fresh retry
- when no attempt existed yet and the action would launch the first checkout

### 3. Shared Portal I18n Lacked Decision-Specific Copy

The new commercial posture needed shared translation keys for:

- resume-existing-attempt CTA copy
- retry-with-new-attempt CTA copy
- first-launch CTA copy
- explicit inline decision detail text

Without that, the new decision clarity would either remain implicit or regress the shared Portal i18n boundary.

## Implemented Fixes

- added `buildBillingCheckoutLaunchDecision(...)` to `sdkwork-router-portal-billing` services
- defined the canonical decision surface as:
  - `resume_existing_attempt`
  - `create_retry_attempt`
  - `create_first_attempt`
- updated the Portal billing page to:
  - derive launch behavior from canonical `payment_attempts`
  - reuse the helper instead of page-local decision branching
  - show explicit provider-launch status copy for reopen versus retry versus first launch
  - render method-card explanation copy describing the current decision
  - label the CTA according to the computed decision
- updated shared Portal i18n registries and `zh-CN` translations to cover the new decision language
- added service-level and source-contract coverage so the decision remains reusable and regression-safe

## Files Touched In This Slice

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/services/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
- `apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `docs/release/CHANGELOG.md`

## Verification Evidence

### Coverage Added For This Slice

- service-level classification coverage for:
  - `resume_existing_attempt`
  - `create_retry_attempt`
  - `create_first_attempt`
- source-contract coverage proving the billing page still references:
  - `buildBillingCheckoutLaunchDecision`
  - `Resume provider checkout`
  - `Retry with new attempt`
  - `Launch provider checkout`
- i18n/source assertions covering the new decision copy in shared Portal text registries

### Green

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## Step 06 Assessment

### Closed In This Slice

- provider checkout launch posture is now explicit, named, and reusable
- the Portal workbench now explains whether it is resuming an existing checkout or creating a fresh retry
- the new decision posture is covered by shared i18n instead of page-local raw text
- the decision rule is now testable outside the page

### Still Open

- the larger checkout workbench shell still depends on compatibility checkout-session structure
- operator settlement and callback rehearsal remain compatibility bridge behavior
- Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98` are not globally closed by this slice alone

## Architecture Writeback Decision

- `docs/架构/*` was intentionally not updated in this slice
- reason: this iteration clarified behavior inside an already-established formal payment-attempt lane and did not change control-plane ownership, API truth, role boundaries, or configuration safety posture

## Score

- 目标与边界: `9 / 10`
- 架构对齐: `13 / 15`
- 工程落地: `14 / 15`
- 测试完整性: `14 / 15`
- 结果验证: `14 / 15`
- 证据与文档: `10 / 10`
- 并行交付与收口: `9 / 10`
- 风险与回滚: `9 / 10`
- 总分: `92 / 100`

## Next Slice Recommendation

1. Reduce remaining compatibility checkout-session dependence in the pending-payment workbench while keeping canonical attempt truth primary.
2. Continue exposing attempt-backed operational detail only when formal backend semantics exist.
3. Keep Step 06 commercialization progress incremental and truthful instead of widening scope into a new payment console.
