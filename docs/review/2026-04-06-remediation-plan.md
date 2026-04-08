# SDKWork API Router Remediation Plan

> **For agentic workers:** execute this plan in small verified slices. Keep using targeted regression tests before touching production code. Do not mark a step complete without fresh command evidence.

**Goal:** close the highest-risk commercial-readiness gaps proven in the current review, starting with panic-prone runtime paths and failing admin quality gates.

**Architecture:** stabilize the system in concentric layers. First remove crash-only behavior from startup and request paths. Then restore admin quality gates so UI/commercial surfaces are deterministic and localizable. Finally harden verification and operational readiness across service and product runtime packages.

**Tech Stack:** Rust, Axum, tower-http, TypeScript, React, PNPM, Cargo

---

### Task 1: Finish the HTTP exposure bootstrap hardening

**Files**

- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `services/gateway-service/src/main.rs`
- Modify: `services/admin-api-service/src/main.rs`
- Modify: `services/portal-api-service/src/main.rs`
- Modify: `crates/sdkwork-api-product-runtime/src/lib.rs`
- Test: `crates/sdkwork-api-interface-http/tests/cors_preflight.rs`

- [x] Add a failing regression test proving invalid CORS allowlist entries should not crash router construction.
- [x] Run the targeted gateway CORS test and confirm the failure mode existed before the fix.
- [x] Add explicit `HttpExposureConfig` injection entrypoints for gateway/admin/portal routers.
- [x] Switch service entrypoints and product runtime listeners to injected exposure config.
- [x] Make the gateway CORS layer ignore invalid header values instead of panicking.
- [x] Re-run targeted gateway CORS regression tests.
- [x] Replace remaining public env-based panic constructors with explicit fallible builders or documented compatibility wrappers.

### Task 2: Remove request-path `expect(...)` from live gateway handlers

**Files**

- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Add/modify tests in: `crates/sdkwork-api-interface-http/tests/*.rs`

- [ ] Inventory live handler `expect(...)` sites and separate them from test-only/internal scaffolding.
- [ ] Pick the first cluster with real operator impact:
- [x] Inventory live handler `expect(...)` sites and separate them from test-only/internal scaffolding.
- [x] Pick the first cluster with real operator impact:
  - `compact_response`
  - thread/run retrieval
  - file/content retrieval
  - upload/fine-tuning/vector-store retrieval helpers
- [x] Write failing HTTP regression tests for the first chosen failure paths (`file_content`, `thread_run_step`).
- [x] Replace the first chosen `expect(...)` sites with typed HTTP error mapping.
- [x] Re-run targeted tests and confirm the service returns controlled 404 responses instead of aborting for the first chosen failure paths.
- [x] Repeat the same TDD cycle for local container/music/video content retrieval and verify unknown resources now return controlled 404 responses.
- [x] Repeat the same TDD cycle for local `response compact` handling and verify invalid empty-model input now returns a controlled 400 invalid-request response.
- [x] Repeat the same TDD cycle for local response retrieve/input-items/delete/cancel handling and verify unknown `response_id` now returns controlled 404 responses in both stateless and stateful fallbacks.
- [x] Repeat the same TDD cycle for local `responses` create and `input_tokens` handling and verify empty-model requests now return controlled 400 invalid-request responses in both stateless and stateful fallbacks, including local stream fallback and no-usage-on-invalid guarantees.
- [x] Repeat the same TDD cycle for local OpenAI chat-completion creation and Anthropic `count_tokens` handling and verify empty-model requests now return controlled 400 compatibility-appropriate error responses in both stateless and stateful fallbacks, including chat stream fallback and no-usage-on-invalid guarantees.
- [x] Repeat the same TDD cycle for Anthropic `messages` and Gemini `generateContent`/`streamGenerateContent`/`countTokens` local fallback handling and verify empty-model requests now return controlled 400 compatibility-appropriate error responses in both stateless and stateful fallbacks, without usage on invalid requests.
- [x] Repeat the same TDD cycle for local OpenAI chat-completion retrieve/update/delete handling and verify unknown `completion_id` now returns controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local OpenAI chat-completion message-list handling and verify unknown `completion_id` now returns controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local conversation retrieve/update/delete handling and verify unknown `conversation_id` now returns controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local conversation item create/list/retrieve/delete handling and verify missing parent conversations or missing item ids now return controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local thread retrieve/update/delete handling and verify unknown `thread_id` now returns controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local thread message create/list/retrieve/update/delete handling and verify missing parent threads or missing message ids now return controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local thread run create/list/retrieve/update/cancel/submit-tool-outputs/steps handling and verify missing parent threads, run ids, or step ids now return controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local file retrieve/delete handling and verify unknown `file_id` now returns controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local vector-store file retrieve/delete handling and verify unknown vector-store `file_id` now returns controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local vector-store file batch retrieve/cancel/files handling and verify unknown batch ids now return controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local model retrieve/delete handling and verify unknown `model_id` now returns controlled 404 responses in the stateless fallback.
- [x] Repeat the same TDD cycle for local batch retrieve/cancel handling and verify unknown `batch_id` now returns controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local vector-store retrieve/update/delete handling and verify unknown `vector_store_id` now returns controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local assistant retrieve/update/delete handling and verify unknown `assistant_id` now returns controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local eval retrieve/update/delete and eval-run/output-item handling and verify missing parent evals, run ids, or output-item ids now return controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local container retrieve/delete and container-file create/list/retrieve/delete/content handling and verify missing parent containers or missing file ids now return controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local music retrieve/delete handling and verify unknown `music_id` now returns controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests; also defer stateful local music content usage capture until the local lookup succeeds.
- [x] Repeat the same TDD cycle for local video retrieve/delete handling and verify unknown `video_id` now returns controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests; also defer stateful local video content usage capture until the local lookup succeeds.
- [x] Repeat the same TDD cycle for local upload part/complete/cancel handling and verify unknown `upload_id` values now return controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local fine-tuning job retrieve/cancel/events/checkpoints/pause/resume handling and checkpoint-permission create/list/delete handling, and verify missing jobs/checkpoints/permissions now return controlled 404 responses in both stateless and stateful fallbacks, without usage on not-found local requests.
- [x] Repeat the same TDD cycle for local file upload create handling and verify blank `purpose` or `filename` values now return controlled 400 invalid-request responses in both stateless and stateful fallbacks, without usage on invalid local requests.
- [x] Repeat the same TDD cycle for local completion create handling and verify blank `model` values now return controlled 400 invalid-request responses in both stateless and stateful fallbacks, without usage on invalid local requests.
- [x] Repeat the same TDD cycle for local thread-and-run create handling and verify blank `assistant_id` values now return controlled 400 invalid-request responses in both stateless and stateful fallbacks, without usage on invalid local requests.
- [x] Repeat the same TDD cycle for local embeddings create handling and verify blank `model` values now return controlled 400 invalid-request responses in both stateless and stateful fallbacks, without usage on invalid local requests; also verify canonical commercial billing holds are released on invalid local embedding requests.
- [x] Repeat the same TDD cycle for local moderations create handling and verify blank `model` values now return controlled 400 invalid-request responses in both stateless and stateful fallbacks, without usage on invalid local requests.
- [x] Repeat the same TDD cycle for local image-generation create handling and verify blank `model` values now return controlled 400 invalid-request responses in both stateless and stateful fallbacks, without usage on invalid local requests.
- [ ] Repeat the same TDD cycle for the remaining request-path `expect(...)` clusters.

### Task 3: Restore admin i18n quality gate module by module

**Files**

- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-pricing/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18n.tsx`
- Add tests under: `apps/sdkwork-router-admin/tests/`

- [x] Add targeted commercial i18n regression coverage.
- [x] Fix commercial metadata and API key group placeholder copy.
- [x] Add targeted rate-limit i18n regression coverage.
- [x] Fix rate-limit route/model placeholder copy.
- [x] Add a targeted pricing-module i18n regression test for the current hotspot placeholders and sample labels.
- [x] Replace raw pricing placeholders and visible enum/sample strings with `t(...)` or structured translated metadata.
- [x] Add the required `zh-CN` keys.
- [x] Re-run `node tests/admin-i18n-coverage.test.mjs` and restore the global admin i18n structural gate to green.

### Task 4: Convert the missing admin translation backlog into managed work

**Files**

- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18n.tsx`
- Add docs: `docs/review/`

- [x] Extract the current failing missing-key list from `admin-i18n-coverage.test.mjs`.
- [x] Group missing keys by module:
  - [x] commercial
  - [x] pricing
  - [x] routing/access
  - [x] billing/settlement
  - [x] marketing/coupon
- [ ] Replace the generated English-valued commercial backfill entries with curated Simplified Chinese translations in separate verified slices.
  - [x] marketing/coupon order-audit copy now lives in `ADMIN_ZH_MARKETING_TRANSLATIONS` with targeted regression coverage in `tests/admin-i18n-marketing-regressions.test.mjs`.
  - [x] billing/settlement/payment/refund hotspot now lives in `ADMIN_ZH_BILLING_SETTLEMENT_TRANSLATIONS` with targeted regression coverage in `tests/admin-i18n-billing-settlement-regressions.test.mjs`.
  - [x] commercial account/admission hotspot now lives in `ADMIN_ZH_COMMERCIAL_ACCOUNT_TRANSLATIONS` with targeted regression coverage in `tests/admin-i18n-commercial-account-regressions.test.mjs`.
  - [x] pricing governance/operator copy now lives in `ADMIN_ZH_PRICING_TRANSLATIONS` with targeted regression coverage in `tests/admin-i18n-pricing-governance-regressions.test.mjs`.
  - [x] pricing module surface/catalog copy now routes option catalogs, lifecycle summaries, charge-unit detail copy, billing-method detail copy, and rounding labels through `ADMIN_ZH_PRICING_TRANSLATIONS` with targeted regression coverage in `tests/admin-i18n-pricing-module-surface.test.mjs`.
  - [x] routing/access policy-group copy now lives in `ADMIN_ZH_ROUTING_ACCESS_TRANSLATIONS` with targeted regression coverage in `tests/admin-i18n-routing-access-regressions.test.mjs`.
  - [x] commercial dashboard summary/order-audit shell copy now lives in `ADMIN_ZH_COMMERCIAL_SURFACE_TRANSLATIONS` with targeted regression coverage in `tests/admin-i18n-commercial-surface-regressions.test.mjs`.
  - [x] commercial ledger/refund/order-audit detail labels now live in `ADMIN_ZH_COMMERCIAL_DETAIL_TRANSLATIONS` with targeted regression coverage in `tests/admin-i18n-commercial-detail-regressions.test.mjs`.
  - [x] apirouter access/routing/snapshot shell copy now lives in `ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS` with targeted regression coverage in `tests/admin-i18n-apirouter-surface-regressions.test.mjs`.
  - [x] apirouter usage analytics shell copy now extends `ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS` with targeted regression coverage in `tests/admin-i18n-apirouter-usage-surface.test.mjs`.
  - [x] apirouter access-detail and routing-impact copy now lives in `ADMIN_ZH_APIROUTER_DETAIL_TRANSLATIONS` with targeted regression coverage in `tests/admin-i18n-apirouter-detail-regressions.test.mjs`.
  - [x] pricing plan/rate workflow copy now extends `ADMIN_ZH_PRICING_TRANSLATIONS` with targeted regression coverage in `tests/admin-i18n-pricing-workflow-regressions.test.mjs`.
  - [x] coupon governance/detail shell copy now extends `ADMIN_ZH_MARKETING_TRANSLATIONS` with targeted regression coverage in `tests/admin-i18n-coupons-governance-regressions.test.mjs`.
  - [x] traffic analytics shell copy now lives in `ADMIN_ZH_TRAFFIC_TRANSLATIONS` with targeted regression coverage in `tests/admin-i18n-traffic-analytics-regressions.test.mjs`.
  - [x] apirouter rate-limit and routing-snapshot tail labels now extend `ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS`, while protocol literals (`sk-router-live-demo`, `Authorization: Bearer {token}`) remain intentionally raw with targeted regression coverage in `tests/admin-i18n-apirouter-tail-policy.test.mjs`.
  - [x] pricing display-unit literals and the settings `Shell` label are now localized with targeted regression coverage in `tests/admin-i18n-pricing-display-shell-regressions.test.mjs`.
  - [x] Re-inventory residual semantic-only English backfill outside the completed hotspot slices:
    - 9 English-valued keys remain in the effective `zh-CN` admin catalog after spread resolution
    - current hit concentration is `sdkwork-router-admin-auth` (3) and `sdkwork-router-admin-apirouter` (3), with the remaining packages each contributing a single intentional literal
    - the remaining residuals are now limited to intentional brand/example/protocol literals: `GitHub`, `Google`, `name@example.com`, `Authorization: Bearer {token}`, `sk-router-live-demo`, `{tenant} / {provider}`, `Ctrl K`, and `4102444800000`
    - `sdkwork-router-admin-apirouter` is reduced to 3 hits / 2 distinct keys, both intentionally preserved protocol/example literals
    - `sdkwork-router-admin-coupons` is reduced to zero effective semantic-English hits in the current inventory
    - `sdkwork-router-admin-traffic` is reduced to zero effective semantic-English hits in the current inventory
    - `sdkwork-router-admin-pricing` is reduced to zero effective semantic-English hits in the current inventory
- [x] Keep the test green between slices instead of allowing the backlog to expand.
- [x] 2026-04-07 recovery slice: normalized the corrupted admin translation source modules and introduced `ADMIN_ZH_RECOVERY_TRANSLATIONS` so 158 newly used payment/control-plane keys are explicitly registered and the structural gate remains green.
- [ ] Replace the English-valued recovery slice for payment method, webhook, refund, and reconciliation surfaces with curated Simplified Chinese translations in verified batches.

### Task 5: Close the verification gap for product runtime and services

**Files**

- Modify docs under: `docs/review/`
- Optionally add CI/build docs under: `docs/operations/` or `docs/getting-started/`

- [x] Define a minimal required verification matrix for changed packages in `docs/review/2026-04-06-rust-verification-matrix.md`.
- [x] Split long-running cargo checks into smaller package gates that can complete reliably in `docs/review/2026-04-06-rust-verification-matrix.md`.
- [x] Document native dependency hot spots such as `libz-ng-sys` in `docs/review/2026-04-06-rust-verification-matrix.md`.
- [x] Add CI caching or package-group guidance so cross-platform verification becomes repeatable.

### Task 6: Continue the broader commercial-system review requested by the user

**Files**

- Add docs under: `docs/review/`
- Modify business crates after evidence-backed review

- [ ] Payment/order/refund/account-history transactional closure review
  - [x] P0 slice 1: harden commercial side-effect consistency for the current SQLite/Postgres path so failed final order writes no longer leave quota or membership mutations behind.
  - [x] Add explicit `delete_quota_policy` and `delete_project_membership` store operations to support commercial rollback on SQLite/Postgres.
  - [x] Add regression coverage proving failed `fulfilled` and `refunded` order writes restore prior commercial state instead of leaving half-applied business effects behind.
  - [x] Continue with canonical account-history/request-settlement linkage for commerce orders.
  - [x] For account-provisioned workspaces, settled/refunded recharge orders now sync canonical account ledger history and advance the commerce reconciliation checkpoint, while recharge orders intentionally continue to leave `request_settlements` empty because they are not request-capture records.
  - [x] Add replay recovery coverage proving a failed canonical account-ledger write leaves the order `fulfilled`, records the payment event as failed with the latest order status, and repairs the canonical account history when the same payment event is replayed.
  - [x] Close refund compensation for coupon rollback evidence when final refunded-order persistence fails.
  - [x] Failed refunded-order persistence now compensates coupon marketing state atomically on SQLite/Postgres by restoring budget/code/redemption back to the pre-refund snapshot, marking the rollback audit as `failed`, and preserving replayability so the same refund event can later complete the rollback cleanly.
- [ ] Coupon concurrency, rollback, and inventory restoration review
  - [x] Inline reclaim expired coupon reservations on the SQLite/Postgres commerce and portal reservation path so stale timed-out reservations no longer block a fresh order or direct portal reservation attempt for the same coupon code.
  - [x] Add regression coverage proving an expired reservation is marked `expired`, budget is rebalanced instead of double-counted, and a fresh reservation can be created immediately on both the commerce order path and `/portal/marketing/coupon-reservations`.
- [ ] Traffic control, failure recovery, monitoring, and auto-failover review
- [ ] Performance/load verification design and benchmarks
- [ ] Cross-platform packaging/runtime review

## Immediate Execution Queue

1. Reclassify the remaining fixed-structure request-path `expect(...)` samples as hygiene debt unless a real failure surface is proven; current review found no meaningful invalid-input or domain-error trigger in the sampled local fallbacks for model list, chat-completion list, conversation create/list, thread create, or image edit/variation.
2. Treat the remaining admin i18n residuals as intentional literals unless product requirements later call for localized brand/example handling.
3. Continue the coupon-domain P1 review with the next stale-state/observability slice: catalog visibility, recovery telemetry, and any remaining retry/concurrency gaps around coupon reservations.
4. Follow with traffic-control/failure-recovery/monitoring review tasks from Task 6.
5. Keep MySQL/libSQL deferred; commercial persistence hardening continues on SQLite/Postgres only until the business closure backlog is done.
