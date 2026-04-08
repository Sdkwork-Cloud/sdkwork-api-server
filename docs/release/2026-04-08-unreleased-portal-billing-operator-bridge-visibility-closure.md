# 2026-04-08 Unreleased Portal Billing Operator Bridge Visibility Closure

## 1. Iteration Context

- Wave / Step: `B / 06`
- Primary mode: `operator-bridge-visibility-closure`
- Current state classification: `green`

## 2. Top 3 Candidate Actions

1. Remove operator-bridge posture from the default checkout method list and rail summary while preserving the existing simulation gate.
2. Rebuild the pending-payment workbench into a new operator-versus-customer split console.
3. Leave the current mixed presentation in place and rely on simulation gating alone.

Action `1` was selected because it closes a visible commercialization gap immediately, without changing backend contracts or widening the Portal billing surface into a new console.

## 3. Actual Changes

- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
  - filtered visible checkout methods so `settle_order` no longer appears in the default workbench method list when simulation mode is off
  - replaced the remaining user-facing `Operator settlement` page copy with `Manual settlement`
  - rebuilt the `Payment rail` panel so it now summarizes formal-first primary rail, selected reference, and payable price instead of hardcoding bridge rows
  - prevented compatibility `manual_lab` rail posture from becoming the default payment-rail summary when simulation mode is not active
- updated shared Portal i18n registries and `zh-CN` translations for the new billing rail summary copy
- added source-contract regressions for:
  - visible checkout method filtering
  - removal of `Operator settlement` from billing page copy
  - new shared i18n coverage for `Manual settlement`

## 4. Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-commercial-api-surface.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-billing-i18n-polish.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `pnpm.cmd --dir apps/sdkwork-router-portal run typecheck`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/*.mjs`

## 5. Architecture / Delivery Impact

- the Portal billing workbench now presents formal payment truth more cleanly by reducing the remaining operator-bridge posture in default user-facing summaries
- user-facing payment copy is more commercially aligned and less likely to imply that manual/operator settlement is the canonical checkout rail
- the slice improves Step 06 commercialization evidence without overstating whole-step completion

## 6. Risks / Limits

- queue-level `Settle order` still remains available in the explicit payment-simulation posture
- provider callback rehearsal still remains available in the explicit payment-simulation posture
- compatibility checkout-session still exists as a bridge / fallback source
- this slice does not add new backend payment contracts or remove the remaining compatibility mutation paths

## 7. Next Entry

1. Continue reducing compatibility bridge ownership inside the pending-payment workbench where formal backend truth already exists.
2. Keep future Portal billing changes tied to real backend semantics only.
3. Continue the Step 06 commercialization lane from the now more formal-first billing payment surface.
