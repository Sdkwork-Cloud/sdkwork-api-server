# 2026-04-08 Unreleased Portal Billing Callback Confirmation Vocabulary Productization

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `callback-confirmation-vocabulary-productization`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Productize the remaining Portal billing replay outcome wording so the surface speaks in payment-confirmation language instead of callback-flow terminology.
2. Leave the callback wording in place because it is technically accurate to provider integrations.
3. Broaden the slice into runtime callback renames.

Action `1` was selected because it closes a visible commercialization gap immediately without widening the slice into runtime or architecture changes.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - rewrote replay outcome feedback from `callback flow` / `provider callback` wording to `payment confirmation`
- updated shared Portal i18n and `zh-CN` translations so provider-specific and provider-generic replay status strings now use payment-confirmation language consistently
- updated Portal billing i18n and product proof so the old callback wording now fails source-contract verification

## 4. Verification

- red first:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- green:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
    - full Portal Node suite returned `221 / 221` passing

## 5. Architecture / Delivery Impact

- Portal billing replay feedback now presents settled, failed, and canceled outcomes as happening after payment confirmation instead of callback flow
- the slice keeps all runtime payment behavior unchanged while improving Step 06 commercialization evidence
- no `docs/架构/*` writeback was required because this iteration changed presentation copy, i18n, and proof only

## 6. Risks / Limits

- `Provider handoff` still remains in billing copy and method labels
- replay action labels still use provider-event oriented `Replaying ...` wording
- manual settlement bridge behavior still remains when simulation posture permits it
- this slice changes product copy, i18n coverage, and proof only; it does not modify backend payment contracts or billing behavior

## 7. Next Entry

1. Continue reducing provider-bridge wording from Portal billing, starting with `Provider handoff`.
2. Keep future billing presentation fixes tied to the real formal backend payment path.
3. Continue the Step 06 commercialization lane from the now more productized payment-confirmation baseline.
