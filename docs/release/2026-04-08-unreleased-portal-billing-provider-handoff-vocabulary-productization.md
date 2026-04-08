# 2026-04-08 Unreleased Portal Billing Provider Handoff Vocabulary Productization

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `provider-handoff-vocabulary-productization`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Productize the remaining `Provider handoff` wording so the Portal billing surface speaks in `Checkout access` language.
2. Leave the runtime-oriented wording in place because it matches the internal action enum.
3. Broaden the slice into runtime enum renames.

Action `1` was selected because it closes a visible commercialization gap immediately without widening the slice into runtime or architecture changes.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - renamed the tenant-facing action label from `Provider handoff` to `Checkout access`
  - rewrote the payment-attempt and payment-method explanatory copy to use `checkout access`
- updated shared Portal i18n and `zh-CN` translations so the new checkout-access wording is fully registered
- updated Portal billing i18n and product proof so the old `Provider handoff` wording now fails source-contract verification

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

- Portal billing now presents the provider checkout capability as `Checkout access` instead of `Provider handoff`
- the slice keeps all runtime payment behavior unchanged while improving Step 06 commercialization evidence
- no `docs/架构/*` writeback was required because this iteration changed presentation copy, i18n, and proof only

## 6. Risks / Limits

- replay action wording still uses provider-event oriented `Replaying ...` copy
- sandbox guidance still retains some provider-event vocabulary
- manual settlement bridge behavior still remains when simulation posture permits it
- this slice changes product copy, i18n coverage, and proof only; it does not modify backend payment contracts or billing behavior

## 7. Next Entry

1. Continue reducing provider-event wording from Portal billing replay actions and guidance.
2. Keep future billing presentation fixes tied to the real formal backend payment path.
3. Continue the Step 06 commercialization lane from the now more productized checkout-access baseline.
