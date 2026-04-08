# 2026-04-08 Unreleased Portal Billing Payment Outcome Sandbox Productization

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `payment-outcome-sandbox-productization`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Productize the remaining provider-event replay wording inside the Portal billing sandbox so the visible workbench speaks in payment-outcome language.
2. Leave the current provider-event wording in place because the sandbox remains a technical tool.
3. Broaden the slice into runtime provider-event contract renames.

Action `1` was selected because it closes a visible commercialization gap immediately without widening the slice into runtime or architecture changes.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - renamed the sandbox badge from `Provider events` to `Payment outcomes`
  - rewrote the sandbox guidance sentence to speak in payment outcomes
  - rewrote in-progress status messages from `Replaying ...` to `Applying ... outcome ...`
  - rewrote the sandbox action buttons from `Replay ... event` to `Apply ... outcome`
- updated shared Portal i18n and `zh-CN` translations so the new payment-outcome wording is fully registered
- updated `apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs` so the broader proof lane now matches the current product wording

## 4. Verification

- red first:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- green:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
  - `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
  - initial full-suite run surfaced one stale proof lane at `220 / 221`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-payment-rails.test.mjs`
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`
    - final full Portal Node suite returned `221 / 221` passing

## 5. Architecture / Delivery Impact

- Portal billing sandbox now presents settlement/failure/cancellation actions through payment-outcome language instead of provider-event replay wording
- the slice keeps all runtime payment behavior unchanged while improving Step 06 commercialization evidence
- no `docs/架构/*` writeback was required because this iteration changed presentation copy, i18n, and proof only

## 6. Risks / Limits

- `Payment event sandbox` and `Event target` still remain as explicit sandbox vocabulary
- `Event signature` still remains visible as a low-level detail label
- manual settlement bridge behavior still remains when simulation posture permits it
- this slice changes product copy, i18n coverage, and proof only; it does not modify backend payment contracts or billing behavior

## 7. Next Entry

1. Continue reducing the remaining sandbox-specific low-level vocabulary from Portal billing where it still leaks into the tenant-facing surface.
2. Keep future billing presentation fixes tied to the real formal backend payment path.
3. Continue the Step 06 commercialization lane from the now more productized payment-outcome sandbox baseline.
