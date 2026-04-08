# 2026-04-08 Unreleased Portal Billing Settlement And Sandbox Posture Productization

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `settlement-and-sandbox-posture-productization`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Productize the remaining settlement-summary and explicit simulation wording in Portal billing so those surfaces read as billing coverage and sandbox tooling instead of operator/callback posture.
2. Remove the payment-event sandbox from Portal billing entirely.
3. Leave the current wording in place and rely on the existing simulation gate alone.

Action `1` was selected because it closes a visible commercialization gap immediately without widening the slice into runtime or architecture changes.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - renamed `Commercial settlement rail` to `Settlement coverage`
  - rewrote the settlement summary description into billing-snapshot language
  - renamed the explicit simulation surface to `Payment event sandbox`
  - relabeled the sandbox badge, selector, placeholder, active rail sentence, and action buttons into sandbox payment-event wording
- updated shared Portal i18n registries and `zh-CN` translations for the new settlement/sandbox vocabulary
- updated billing product/workspace/i18n source-contract tests so the old operator/callback wording now fails proof

## 4. Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-workspace.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## 5. Architecture / Delivery Impact

- the billing workspace now presents settlement facts through product-facing billing language
- the explicit simulation panel is still available when enabled, but it is now clearly framed as sandbox tooling rather than a callback-rehearsal console
- the slice improves Step 06 commercialization evidence without overstating whole-step completion

## 6. Risks / Limits

- explicit payment-event sandbox behavior still remains in runtime
- manual settlement bridge behavior still remains when simulation posture permits it
- this slice changes product copy, i18n coverage, and proof only; it does not remove simulation behavior or modify backend payment contracts

## 7. Next Entry

1. Continue reducing operator-oriented wording from Portal billing.
2. Keep future billing presentation fixes tied to the real formal backend payment path.
3. Continue the Step 06 commercialization lane from the now more productized settlement/sandbox baseline.
