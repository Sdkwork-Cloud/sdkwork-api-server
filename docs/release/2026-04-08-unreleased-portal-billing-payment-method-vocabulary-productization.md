# 2026-04-08 Unreleased Portal Billing Payment Method Vocabulary Productization

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `payment-method-vocabulary-productization`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Productize the remaining Portal billing `rail` wording so the workbench reads in payment-method and sandbox-target language instead of routing jargon.
2. Leave the current `rail` wording in place because it is technically accurate.
3. Remove the affected summary and sandbox labels entirely.

Action `1` was selected because it closes a visible commercialization gap immediately without widening the slice into runtime or architecture changes.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - renamed `Payment rail` to `Payment method`
  - renamed `Primary rail` to `Primary method`
  - renamed `Event rail` to `Event target`
  - renamed `Choose event rail` to `Choose event target`
  - rewrote remaining guidance/history/sandbox sentences away from `rail` terminology
- updated shared Portal i18n and `zh-CN` translations so the new payment-method vocabulary is fully registered, including the previously missing `Payment method` shared key
- updated Portal billing i18n and product proof so the old `rail` wording now fails source-contract verification

## 4. Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## 5. Architecture / Delivery Impact

- Portal billing now presents payment summary, history, and sandbox selection through product-facing payment-method language
- the slice keeps all runtime payment behavior unchanged while improving Step 06 commercialization evidence
- no `docs/架构/*` writeback was required because this iteration changed presentation copy, i18n, and proof only

## 6. Risks / Limits

- explicit sandbox behavior still remains in runtime
- provider callback wording still remains in some outcome messages
- manual settlement bridge behavior still remains when simulation posture permits it
- this slice changes product copy, i18n coverage, and proof only; it does not modify backend payment contracts or billing behavior

## 7. Next Entry

1. Continue reducing provider-bridge and callback-oriented language from Portal billing.
2. Keep future billing presentation fixes tied to the real formal backend payment path.
3. Continue the Step 06 commercialization lane from the now more productized payment-method baseline.
