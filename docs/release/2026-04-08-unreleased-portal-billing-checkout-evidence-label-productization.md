# 2026-04-08 Unreleased Portal Billing Checkout Evidence Label Productization

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `checkout-evidence-label-productization`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Productize the remaining checkout-evidence labels so the Portal billing surface renders `Checkout reference` and `QR code content`.
2. Leave the existing low-level labels visible because they mirror backend field names.
3. Broaden the slice into cross-stack contract renames around `session_reference` and `qr_code_payload`.

Action `1` was selected because it closes a visible commercialization gap immediately without widening the slice into runtime or architecture changes.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - replaced `Session reference` with `Checkout reference`
  - replaced `QR payload` with `QR code content`
- updated shared Portal i18n and `zh-CN` translations so the new checkout-evidence labels are fully registered
- updated Portal billing, product, and payment-rails proof so the new evidence labels are part of the active source contract

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

- Portal billing checkout-method evidence now uses more product-facing labels for the reference and QR evidence rows
- the slice keeps `session_reference`, `qr_code_payload`, repository composition, and payment runtime behavior unchanged
- no `docs/架构/*` writeback was required because this iteration changed presentation copy, i18n, and proof only

## 6. Risks / Limits

- other low-level billing metadata may still need future productization
- this slice changes product copy, i18n coverage, and proof only; it does not modify backend payment contracts or billing behavior

## 7. Next Entry

1. Continue reducing remaining low-level billing metadata wording where it still leaks into tenant-facing checkout evidence.
2. Keep future billing presentation fixes tied to the real formal backend payment path.
3. Continue the Step 06 commercialization lane from the now more readable checkout-evidence baseline.
