# 2026-04-08 Unreleased Step 06 Admin Portal Test Contract Hardening

## 1. Iteration Context

- Wave / Step: `Wave B / Step 06`
- Primary mode: `verification-contract-hardening`
- Current state classification: `in_progress`

## 2. Top 3 Candidate Actions

1. Repair the active Step 06 admin/portal frontend contract failures before expanding scope.
2. Leave the frontend contract failures unresolved and continue only on the release-truth lane.
3. Expand into broader portal or admin product polishing before the current red proofs are stabilized.

Action `1` was selected because both red proofs were inside the currently unlocked Step 06 admin/portal commercialization lane and were blocking trustworthy continuation.

## 3. Actual Changes

- updated `apps/sdkwork-router-admin/tests/admin-i18n-coverage.test.mjs`
  - replaced the brittle direct `typescript` import with a readable-entry resolver over the local pnpm layout
  - probes readability with a real file-open check instead of `accessSync`, which produced a false positive for the unreadable admin-local TypeScript runtime in this Windows sandbox
  - falls back to another readable workspace-owned pnpm copy only when the preferred admin-local entry is not readable
- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/pages/index.tsx`
  - restored the dual-level redeem history contract by keeping the detail table slot `portal-redeem-reward-history-table` and adding a page-level wrapper slot `portal-redeem-history-table`
  - preserved the product-facing `Reward history` heading while keeping `Redeem history` evidence copy available for the higher-level workspace contract
  - switched the history summary line to shared i18n keys so the redeemed / rolled-back counts can localize through the existing portal translation system
- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
  - added the new count-summary translation key
- updated `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`
  - added the Simplified Chinese translation for the redeemed / rolled-back count-summary key

## 4. Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-admin/tests/admin-architecture.test.mjs apps/sdkwork-router-admin/tests/admin-commercial-workbench.test.mjs apps/sdkwork-router-admin/tests/admin-i18n-coverage.test.mjs`
- `node --test --experimental-test-isolation=none apps/sdkwork-router-portal/tests/portal-architecture.test.mjs apps/sdkwork-router-portal/tests/portal-auth-parity.test.mjs apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs apps/sdkwork-router-portal/tests/portal-admin-workspace-polish.test.mjs apps/sdkwork-router-portal/tests/portal-account-workbench.test.mjs apps/sdkwork-router-portal/tests/portal-account-history-i18n-polish.test.mjs apps/sdkwork-router-portal/tests/portal-account-posture-i18n-polish.test.mjs apps/sdkwork-router-portal/tests/portal-api-key-i18n.test.mjs`

## 5. Architecture / Delivery Impact

- Step 06 admin and portal frontend proof now tolerates the current pnpm workspace layout and Windows sandbox readability limits instead of assuming a directly readable app-local TypeScript runtime
- the redeem workspace now exposes both the page-level history slot and the table-level reward-history slot, which removes a real contract split between two active portal verification suites
- portal history summary copy is now routed through shared i18n data instead of relying on an untranslated inline sentence

## 6. Risks / Limits

- the default isolated `node --test` mode still reproduces `spawn EPERM` in this environment; the documented non-isolated runner remains required for frontend verification
- the admin i18n coverage helper now depends on at least one readable workspace-local TypeScript runtime copy being present under pnpm-managed dependencies
- Step 06 overall control-plane and commercialization closure is still open beyond these frontend contract recoveries

## 7. Next Entry

1. Continue the highest-value open Step 06 closure lane after the admin/portal frontend verification surface is stable again.
2. Keep release-truth notes aligned with the sandbox-specific Node test runner constraint until a hosted or unrestricted run proves otherwise.
