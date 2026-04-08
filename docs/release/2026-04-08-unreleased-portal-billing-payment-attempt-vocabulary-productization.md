# 2026-04-08 Unreleased Portal Billing Payment Attempt Vocabulary Productization

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `payment-attempt-vocabulary-productization`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Productize the visible checkout-history wording so the tenant-facing Portal billing surface renders `Checkout attempts` and related checkout-attempt guidance instead of `Payment attempts`.
2. Leave the payment-attempt wording visible because the runtime still consumes canonical payment-attempt contracts.
3. Broaden the slice into cross-stack contract renames around payment-attempt identities.

Action `1` was selected because it closes a visible commercialization gap immediately without widening the slice into runtime or architecture changes.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - replaced `Payment attempts` with `Checkout attempts`
  - replaced empty-state and guidance copy so the workbench now says `fresh checkout attempt` and `No {provider} checkout attempt exists yet`
  - replaced the failed-payment summary detail with checkout-attempt wording
- updated shared Portal i18n and `zh-CN` translations so the new checkout-attempt wording is fully registered
- updated Portal billing, product, and commercial-api proof so the new checkout-attempt wording is part of the active source contract

## 4. Verification

- red first:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- green:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
  - `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
    - full Portal Node suite returned `221 / 221` passing

## 5. Architecture / Delivery Impact

- Portal billing checkout-history surfaces now use checkout-attempt product wording where they previously exposed payment-attempt jargon
- the slice keeps `payment_attempt_id`, canonical payment-attempt payloads, launch decisions, and runtime checkout behavior unchanged
- no `docs/架构/*` writeback was required because this iteration changed presentation copy, i18n, and proof only

## 6. Risks / Limits

- other low-level billing wording may still need future productization
- this slice changes product copy, i18n coverage, and proof only; it does not modify backend payment contracts or checkout behavior

## 7. Next Entry

1. Continue reducing remaining low-level billing wording where it still leaks into tenant-facing checkout and finance surfaces.
2. Keep future billing presentation fixes tied to the real formal backend payment path.
3. Continue the Step 06 commercialization lane from the now more productized Portal checkout-history baseline.
