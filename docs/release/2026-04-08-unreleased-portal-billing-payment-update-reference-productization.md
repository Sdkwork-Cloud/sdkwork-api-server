# 2026-04-08 Unreleased Portal Billing Payment Update Reference Productization

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `payment-update-reference-productization`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Productize the visible Portal billing payment-history label so the tenant-facing surface renders `Payment update reference` instead of `Provider event`.
2. Leave `Provider event` visible because it mirrors the stored field name.
3. Broaden the slice into cross-stack contract renames around `provider_event_id`.

Action `1` was selected because it closes a visible commercialization gap immediately without widening the slice into runtime or architecture changes.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - replaced the payment-history table header `Provider event` with `Payment update reference`
- updated shared Portal i18n and `zh-CN` translations so the new payment-history label is fully registered
- updated Portal payment-history, product, and billing-i18n proof so the new payment-history wording is part of the active source contract

## 4. Verification

- red first:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- green:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-history.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
    - full Portal Node suite returned `221 / 221` passing

## 5. Architecture / Delivery Impact

- Portal billing payment-history surfaces now use billing-reference product wording where they previously exposed provider-event jargon
- the slice keeps `provider_event_id`, payment-history row contracts, repository composition, and payment runtime behavior unchanged
- no `docs/架构/*` writeback was required because this iteration changed presentation copy, i18n, and proof only

## 6. Risks / Limits

- other low-level billing wording may still need future productization
- this slice changes product copy, i18n coverage, and proof only; it does not modify backend payment contracts or billing behavior

## 7. Next Entry

1. Continue reducing remaining low-level billing wording where it still leaks into tenant-facing finance surfaces.
2. Keep future billing presentation fixes tied to the real formal backend payment path.
3. Continue the Step 06 commercialization lane from the now more productized Portal payment-history baseline.
