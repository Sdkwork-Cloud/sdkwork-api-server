# 2026-04-08 Unreleased Portal Billing Checkout Retry / Reopen Decision Clarity

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `checkout-retry-reopen-decision-clarity`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Formalize canonical provider checkout reuse-versus-retry behavior and make the Portal workbench explain the chosen path.
2. Force every provider checkout click to create a fresh payment attempt.
3. Leave the current inline decision logic implicit and undocumented.

Action `1` was selected because the backend already exposes canonical attempt posture, and the Portal should surface that truth directly instead of making users guess what the CTA will do.

## 3. Actual Changes

- added `buildBillingCheckoutLaunchDecision(...)` to `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/services/index.ts`
  - returns `resume_existing_attempt`, `create_retry_attempt`, or `create_first_attempt`
- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - provider checkout launch now uses the shared decision helper
  - loading/status copy now distinguishes reopening an existing provider checkout from creating a fresh retry attempt
  - method-card detail now explains the active launch posture
  - CTA labels now match the computed decision
- updated shared Portal i18n registries and `zh-CN` translations so the new decision copy stays inside the common localization boundary
- added source-contract and service classification coverage for the new decision logic

## 4. Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## 5. Architecture / Delivery Impact

- the Portal billing workbench now exposes a commercial-grade explanation for provider checkout launch behavior instead of leaving a financially relevant action implicit
- canonical payment-attempt posture now drives both CTA wording and decision detail copy
- this slice improves Step 06 Portal closure evidence without overstating whole-step completion

## 6. Risks / Limits

- compatibility checkout-session still anchors the broader panel shell
- operator settlement and provider callback rehearsal are still intentional bridge behavior
- this slice does not add new provider rails or new backend payment semantics
- this slice does not by itself close Step 06 `8.3 / 8.6 / 91 / 95 / 97 / 98`

## 7. Next Entry

1. Continue reducing compatibility checkout-session dependence in the Portal pending-payment workbench.
2. Keep future attempt-level UX changes tied to formal backend semantics only.
3. Continue the Step 06 commercialization lane from the now-explicit provider checkout decision surface.
