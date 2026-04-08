# 2026-04-08 Unreleased Portal Billing Payment Journey Copy Productization

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `payment-journey-copy-productization`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Productize the remaining billing and recharge payment-journey copy so users see checkout-workbench and checkout-completion language instead of settlement/operator wording.
2. Remove the remaining simulation/operator payment surfaces from Portal billing entirely.
3. Leave the current wording in place and rely only on the previous action-boundary slice.

Action `1` was selected because it closes a visible commercialization gap immediately without widening the slice into backend or structural Portal changes.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - pointed checkout-session and selected-order guidance to the checkout workbench
  - replaced `Settle a subscription order` membership guidance with `Complete a subscription checkout`
  - productized pending / failed / payment-history descriptions away from settlement/operator terminology
- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx`
  - renamed `Open billing queue` to `Open billing workbench`
  - aligned membership activation wording with the billing checkout journey
- updated shared Portal i18n registries, `zh-CN` translations, and source-contract tests for the new payment-journey wording
- relaxed the remaining billing-i18n regex so the proof lane matches the real multiline / trailing-comma `t(...)` formatter output

## 4. Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## 5. Architecture / Delivery Impact

- the Portal billing and recharge journey now describes the intended tenant checkout path more directly
- user-facing copy is less likely to imply that settlement or operator review is the default customer flow
- the slice improves Step 06 commercialization evidence without overstating whole-step completion

## 6. Risks / Limits

- explicit bridge/manual actions still remain inside billing
- provider callback rehearsal still remains available in explicit simulation posture
- compatibility checkout-session still exists as a bridge / fallback source
- this slice changes product copy and guidance only; it does not remove runtime bridge behavior

## 7. Next Entry

1. Continue reducing operator-oriented wording from the user-facing Portal billing journey.
2. Keep future billing presentation changes tied to the real formal backend payment path.
3. Continue the Step 06 commercialization lane from the now more productized payment-journey copy baseline.
