# 2026-04-08 Unreleased Portal Billing Verification Method Display Productization

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `verification-method-display-productization`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Productize raw checkout-method verification strategy values so the Portal billing surface renders readable verification-method labels.
2. Leave the raw strategy values visible because they mirror canonical backend semantics.
3. Broaden the slice into contract renames around `webhook_verification`.

Action `1` was selected because it closes a visible commercialization gap immediately without widening the slice into runtime or architecture changes.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - added a display-only verification-label mapping for checkout-method evidence
  - replaced direct rendering of `method.webhook_verification` with readable labels such as `Manual confirmation`, `Stripe signature check`, and `WeChat Pay RSA-SHA256 check`
- updated shared Portal i18n and `zh-CN` translations so the new verification-method display labels are fully registered
- updated Portal billing, product, and payment-rails proof so the new readable verification labels are part of the active source contract

## 4. Verification

- red first:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
- green:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
  - `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
    - full Portal Node suite returned `221 / 221` passing

## 5. Architecture / Delivery Impact

- Portal billing checkout-method evidence now presents readable verification-method labels instead of raw strategy codes
- the slice keeps all runtime payment behavior and canonical commerce payloads unchanged while improving Step 06 commercialization evidence
- no `docs/架构/*` writeback was required because this iteration changed presentation copy, i18n, and proof only

## 6. Risks / Limits

- unknown future verification strategy values still rely on display fallback and may need explicit product mappings if they surface
- this slice changes product copy, i18n coverage, and proof only; it does not modify backend payment contracts or billing behavior

## 7. Next Entry

1. Continue reducing remaining low-level billing metadata wording where it still leaks into tenant-facing checkout evidence.
2. Keep future billing presentation fixes tied to the real formal backend payment path.
3. Continue the Step 06 commercialization lane from the now more readable verification-method baseline.
