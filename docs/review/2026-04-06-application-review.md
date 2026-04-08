# 2026-04-06 Application Review

## Scope

This review round focused on evidence-backed defects already reproducible in the current workspace:

- admin web i18n quality gates
- gateway/admin/portal HTTP exposure bootstrap paths
- gateway runtime panic inventory sampling
- service/product runtime compilation confidence

The goal of this document is to capture what is already proven, what was fixed in this round, and what still blocks commercial-grade readiness.

## Verified This Round

### Implemented fixes

1. Admin commercial route metadata and API key group color placeholder now go through admin i18n helpers.
2. Rate-limit dialog route/model placeholders now go through admin i18n helpers.
3. Pricing-plan and pricing-rate sample/status copy now go through `t(...)` instead of raw literals.
4. The admin i18n catalog now has an explicit commercial backfill map covering the previously missing commercial/routing/billing/settlement keys, which restores the admin i18n structural quality gate to green.
5. Gateway CORS origin parsing now skips invalid header values instead of crashing while building the gateway router.
6. Service and product runtime entry paths now have explicit `HttpExposureConfig` injection hooks so they can reuse already parsed config instead of reparsing process env in the router constructor.
7. Unknown local file-content requests now return a controlled 404 OpenAI error envelope instead of relying on panic-prone `expect(...)`.
8. Unknown thread run-step retrievals now return a controlled 404 OpenAI error envelope instead of relying on panic-prone `expect(...)`.
9. Unknown local container-file content requests now return a controlled 404 OpenAI error envelope instead of incorrectly returning synthetic bytes.
10. Unknown local music content requests now return a controlled 404 OpenAI error envelope instead of incorrectly returning synthetic bytes.
11. Unknown local video content requests now return a controlled 404 OpenAI error envelope instead of incorrectly returning synthetic bytes.
12. Local response compaction now rejects an empty model with a controlled 400 OpenAI invalid-request envelope instead of relying on panic-prone `expect(...)`.
13. Unknown local response retrieve/input-items/delete/cancel requests now return controlled 404 OpenAI error envelopes instead of incorrectly returning synthetic success payloads.
14. Local `responses` create and `input_tokens` fallbacks now reject empty models with a controlled 400 OpenAI invalid-request envelope in both stateless and stateful routers, including local stream fallback, without recording usage on invalid requests.
15. Local OpenAI chat-completion fallbacks now reject empty models with a controlled 400 OpenAI invalid-request envelope in both stateless and stateful routers, including local stream fallback, without recording usage on invalid requests.
16. Anthropic `count_tokens` local fallbacks now reject empty models with a controlled 400 Anthropic invalid-request envelope in both stateless and stateful routers instead of panicking.
17. Anthropic `messages` and Gemini `generateContent`/`streamGenerateContent`/`countTokens` local fallbacks now reject empty models with controlled provider-specific 400 envelopes in both stateless and stateful routers, and invalid local fallback requests no longer create usage records before returning those errors.
18. Gateway/admin/portal routers now expose explicit `try_*_router()` fallible builders, so malformed HTTP exposure env can surface as typed startup errors instead of forcing callers through panic-only construction paths.
19. Unknown local chat-completion message-list requests now return controlled 404 OpenAI error envelopes in both stateless and stateful routers instead of incorrectly returning synthetic success payloads, and the stateful fallback no longer records usage before the local lookup succeeds.
20. Unknown local chat-completion retrieve/update/delete requests now return controlled 404 OpenAI error envelopes in both stateless and stateful routers instead of incorrectly returning synthetic success payloads, and the stateful fallbacks no longer record usage before those local lookups succeed.
21. Unknown local conversation retrieve/update/delete requests now return controlled 404 OpenAI error envelopes in both stateless and stateful routers instead of incorrectly returning synthetic success payloads, and the stateful fallbacks no longer record usage before those local lookups succeed.
22. Unknown local conversation item create/list requests now return controlled 404 OpenAI error envelopes when the parent conversation is missing, and unknown conversation item retrieve/delete requests now return controlled 404 envelopes when the item is missing; the stateful fallbacks no longer record usage before those local checks succeed.
23. Unknown local thread retrieve/update/delete requests now return controlled 404 OpenAI error envelopes in both stateless and stateful routers instead of incorrectly returning synthetic success payloads, and the stateful fallbacks no longer record usage before those local lookups succeed.
24. Unknown local thread message create/list requests now return controlled 404 OpenAI error envelopes when the parent thread is missing, and unknown thread message retrieve/update/delete requests now return controlled 404 envelopes when the local message is missing; the stateful fallbacks no longer record usage before those local checks succeed.
25. Unknown local thread run create/list requests now return controlled 404 OpenAI error envelopes when the parent thread is missing, unknown run retrieve/update/cancel/submit-tool-outputs/steps-list requests now return controlled 404 envelopes when the local run is missing, and stateful run/step fallbacks no longer record usage before the local run or step lookup succeeds.
26. Unknown local file retrieve/delete requests now return controlled 404 OpenAI error envelopes instead of incorrectly returning synthetic success payloads, and the stateful file retrieve/delete fallbacks no longer record usage before the local file lookup succeeds.
27. Unknown local vector-store file retrieve/delete requests now return controlled 404 OpenAI error envelopes instead of incorrectly returning synthetic success payloads, and the stateful vector-store file retrieve/delete fallbacks no longer record usage before the local vector-store file lookup succeeds.
28. Unknown local vector-store file batch retrieve/cancel/files-list requests now return controlled 404 OpenAI error envelopes instead of incorrectly returning synthetic success payloads, and the stateful vector-store batch fallbacks no longer record usage before the local batch lookup succeeds.
29. Unknown local model retrieve/delete requests now return controlled 404 OpenAI error envelopes instead of incorrectly returning synthetic success payloads in the stateless gateway fallback.
30. Unknown local batch retrieve/cancel requests now return controlled 404 OpenAI error envelopes instead of incorrectly returning synthetic success payloads, and the stateful batch fallbacks no longer record usage before the local batch lookup succeeds.
31. Unknown local vector-store retrieve/update/delete requests now return controlled 404 OpenAI error envelopes instead of incorrectly returning synthetic success payloads, and the stateful vector-store fallbacks no longer record usage before the local vector-store lookup succeeds.
32. Unknown local assistant retrieve/update/delete requests now return controlled 404 OpenAI error envelopes instead of incorrectly returning synthetic success payloads, and the stateful assistant fallbacks no longer record usage before the local assistant lookup succeeds.
33. Unknown local eval retrieve/update/delete requests now return controlled 404 OpenAI error envelopes, missing parent evals now stop local eval-run list/create fallbacks with controlled 404 responses, unknown eval runs or output items now return controlled 404 envelopes, and the stateful eval fallbacks no longer record usage before those local checks succeed.
34. Unknown local container retrieve/delete requests now return controlled 404 OpenAI error envelopes, missing parent containers now stop local container-file create/list fallbacks with controlled 404 responses, unknown container-file retrieve/delete/content requests now return controlled 404 envelopes, and the stateful container fallbacks no longer record usage before those local checks succeed.
35. Unknown local music retrieve/delete requests now return controlled 404 OpenAI error envelopes instead of incorrectly returning synthetic success payloads, and the stateful music retrieve/delete/content fallbacks no longer record usage before those local checks succeed.
36. Unknown local video retrieve/delete requests now return controlled 404 OpenAI error envelopes instead of incorrectly returning synthetic success payloads, and the stateful video retrieve/delete/content fallbacks no longer record usage before those local checks succeed.
37. Missing local upload sessions now return controlled 404 OpenAI error envelopes for upload-part create, upload complete, and upload cancel fallbacks instead of incorrectly returning synthetic success payloads, and the stateful upload-part/complete/cancel fallbacks no longer record usage before those local checks succeed.
38. Missing local fine-tuning jobs now return controlled 404 OpenAI error envelopes for retrieve/cancel/events/checkpoints/pause/resume fallbacks, missing local fine-tuning checkpoints now return controlled 404 envelopes for checkpoint-permission create/list fallbacks, missing checkpoint permissions now return controlled 404 envelopes for permission delete fallbacks, and the stateful fine-tuning fallbacks no longer record usage before those local checks succeed.
39. Local file upload create fallbacks now reject blank `purpose` and `filename` values with controlled 400 OpenAI invalid-request envelopes in both stateless and stateful routers, and the stateful fallback no longer records usage before local validation succeeds.
40. Local completion create fallbacks now reject blank `model` values with controlled 400 OpenAI invalid-request envelopes in both stateless and stateful routers, and the stateful fallback no longer records usage before local validation succeeds.
41. Local thread-and-run create fallbacks now reject blank `assistant_id` values with controlled 400 OpenAI invalid-request envelopes in both stateless and stateful routers, and the stateful fallback no longer records usage before local validation succeeds.
42. Local embeddings create fallbacks now reject blank `model` values with controlled 400 OpenAI invalid-request envelopes in both stateless and stateful routers, the stateful fallback no longer records usage before local validation succeeds, and invalid local requests now release canonical commercial billing holds instead of leaving them open.
43. Local moderations create fallbacks now reject blank `model` values with controlled 400 OpenAI invalid-request envelopes in both stateless and stateful routers, and the stateful fallback no longer records usage before local validation succeeds.
44. Local image-generation create fallbacks now reject blank `model` values with controlled 400 OpenAI invalid-request envelopes in both stateless and stateful routers, and invalid local image-generation requests do not create usage records.
45. Commercial order-audit coupon copy now overrides English backfill entries through a dedicated `ADMIN_ZH_MARKETING_TRANSLATIONS` slice, so the `zh-CN` catalog covers coupon evidence-chain, rollback timeline, and coupon-related empty-state copy with curated Simplified Chinese operator wording.
46. Commercial settlement, refund, and payment-audit operator copy now overrides English backfill entries through a dedicated `ADMIN_ZH_BILLING_SETTLEMENT_TRANSLATIONS` slice, so the `zh-CN` catalog covers settlement explorer, settlement ledger, refund timeline, order payment/refund audit, payment evidence timeline, and related empty states with curated Simplified Chinese wording.
47. Commercial account posture and admission-readiness operator copy now overrides English backfill entries through a dedicated `ADMIN_ZH_COMMERCIAL_ACCOUNT_TRANSLATIONS` slice, so the `zh-CN` catalog covers commercial account summary cards, held-balance/admission facts, and account posture/detail copy with curated Simplified Chinese wording.
48. Pricing-governance operator copy now overrides English backfill entries through a dedicated `ADMIN_ZH_PRICING_TRANSLATIONS` slice, so the `zh-CN` catalog covers commercial pricing-governance summary copy, pricing-module operator guidance, gateway access pricing-posture guidance, and the pricing-rate empty-state prompt with curated Simplified Chinese wording.
49. Routing/access policy-group operator copy now overrides English backfill entries through a dedicated `ADMIN_ZH_ROUTING_ACCESS_TRANSLATIONS` slice, so the `zh-CN` catalog covers routing-profile descriptions, API key group policy guidance, reusable routing-profile empty-state copy, and bound-group routing posture descriptions with curated Simplified Chinese wording.
50. A package-level Rust verification matrix now lives in `docs/review/2026-04-06-rust-verification-matrix.md`, documenting the minimal required gate set, package-split `cargo check` strategy, and the `libz-ng-sys` native compile hotspot that previously caused monolithic verification timeouts.
51. Rust verification automation now lives in `scripts/check-rust-verification-matrix.mjs`, so the package-group matrix is executable instead of remaining a manual docs-only checklist.
52. A GitHub Actions workflow now lives in `.github/workflows/rust-verification.yml`, using Rust cache plus package-group fan-out to make the documented service/runtime verification matrix repeatable in CI.
53. Pricing module option catalogs, lifecycle sync summaries, and rounding labels now route through translated builders instead of embedding raw English arrays or runtime-only interpolated English summary strings.
54. The `zh-CN` pricing slice now overrides pricing module stat-card copy, charge-unit and billing-method detail catalogs, and lifecycle summary templates with dedicated Simplified Chinese operator wording.
55. Commercial workspace summary and order-audit shell copy now overrides English backfill entries through a dedicated `ADMIN_ZH_COMMERCIAL_SURFACE_TRANSLATIONS` slice, so the `zh-CN` catalog covers summary facts, primary pricing hints, order-audit loading/empty-state shell copy, and reconciliation footer wording with curated Simplified Chinese operator text.
56. API router access, routing-profile, and compiled-snapshot shell copy now overrides English backfill entries through a dedicated `ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS` slice, so the `zh-CN` catalog covers API key group workflows, routing-profile creation/template copy, compiled snapshot evidence summaries, and related empty states with curated Simplified Chinese operator wording.
57. API router usage analytics shell copy now extends `ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS`, so `GatewayUsagePage` localizes billing-event analytics cards, routing-evidence summaries, multimodal signal tiles, pricing-posture facts, recent billing-event empty states, and export/reporting shell copy with curated Simplified Chinese operator wording.
58. Commercial ledger, refund, settlement, and order-audit detail labels now override English backfill entries through a dedicated `ADMIN_ZH_COMMERCIAL_DETAIL_TRANSLATIONS` slice, so the `zh-CN` catalog covers ledger/refund table headers, investigation actions, order-detail field labels, evidence fallback labels, and settlement amount summary copy with curated Simplified Chinese operator wording.
59. API router access-detail and routing-impact copy now override English backfill entries through a dedicated `ADMIN_ZH_APIROUTER_DETAIL_TRANSLATIONS` slice, so the `zh-CN` catalog covers commercial governance facts, API key group detail labels, routing-impact evidence summaries, compiled-snapshot detail copy, and related drawer/detail descriptions with curated Simplified Chinese operator wording.
60. Pricing plan/rate workflow copy now extends `ADMIN_ZH_PRICING_TRANSLATIONS`, so the `zh-CN` catalog covers pricing governance shell text, lifecycle-convergence guidance, plan/rate composer field labels, workflow actions, and save-state verbs with curated Simplified Chinese operator wording.
61. Coupon governance/detail shell copy now extends `ADMIN_ZH_MARKETING_TRANSLATIONS`, so the `zh-CN` catalog covers template/campaign/budget/code governance actions, lifecycle status labels, compatibility guidance, and coupon governance summary cards with curated Simplified Chinese operator wording.
62. Traffic analytics shell copy now lives in `ADMIN_ZH_TRAFFIC_TRANSLATIONS`, so the `zh-CN` catalog covers billing-event summary cards, group/capability/accounting spotlight empty states, routing-detail fallback labels, billing summary descriptions, and traffic inspection analytics copy with curated Simplified Chinese operator wording.
63. API router rate-limit and routing-snapshot tail labels now extend `ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS`, so the `zh-CN` catalog localizes the remaining UI-facing `Window`/`Policies`/`Live windows`/`Manage routing profiles`/`Snapshot evidence` labels while intentionally preserving protocol/example literals such as `Authorization: Bearer {token}` and `sk-router-live-demo`.
64. Pricing display-unit literals and the settings `Shell` label are now localized, so `commercialPricing.ts` no longer falls back to raw `USD / ...` unit strings or `{count} x {unit}` scaffolding, and the settings center no longer exposes the English `Shell` group label in `zh-CN`.
65. Commercial settlement now snapshots quota and membership state before side effects, so if the final order write fails the current SQLite/Postgres path restores prior quota policy and project membership instead of leaving the order stuck in `pending_payment` with fulfilled side effects already applied.
66. Commercial refund now restores the pre-refund quota policy when the final refunded-order write fails on the current SQLite/Postgres path, so failed refund persistence no longer silently returns quota while the order remains `fulfilled`.
67. The storage contract now exposes explicit `delete_quota_policy` and `delete_project_membership` operations, giving the SQLite/Postgres backends a concrete rollback primitive for commercial compensation work while MySQL/libSQL remain out of scope for this P0 slice.
68. Portal recharge settlement and refund now sync canonical account history for provisioned workspace accounts by issuing or refunding commerce-order credits through the billing kernel after the order reaches its final persisted state, and then advancing the account-commerce reconciliation checkpoint.
69. Canonical portal account-history views now expose recharge order settlement and refund through the existing `ledger` history surface while intentionally keeping `request_settlements` empty for those orders, because recharge purchases are account-funding events rather than request-capture settlements.
70. If canonical account-ledger sync fails after a recharge order has already reached `fulfilled`, the payment event is now persisted as failed with `order_status_after = fulfilled`, and replaying the same payment event repairs the canonical account history instead of getting stuck behind the already-final order state.
71. If a refunded recharge order fails on the final order write after coupon rollback already executed, the current SQLite/Postgres path now compensates marketing budget/code/redemption atomically back to the pre-refund snapshot, marks the rollback evidence as `failed`, and still allows the same refund event to replay later and complete the rollback cleanly.
72. Commerce quote/order creation and the portal marketing reservation API now reclaim expired coupon reservations inline for the current SQLite/Postgres path, so a timed-out stale reservation no longer leaves the coupon code hidden or the reserved budget double-counted when the same project retries immediately.

### Fresh verification evidence

- `node tests/admin-i18n-commercial-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-commercial-account-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-pricing-governance-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-routing-access-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-billing-settlement-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-marketing-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-coupons-governance-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-traffic-analytics-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-apirouter-tail-policy.test.mjs`
  - PASS
- `node tests/admin-i18n-pricing-display-shell-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-pricing-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-rate-limit-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-commercial-surface-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-apirouter-surface-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-apirouter-usage-surface.test.mjs`
  - PASS
- `node tests/admin-i18n-apirouter-detail-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-pricing-workflow-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-commercial-detail-regressions.test.mjs`
  - PASS
- `node tests/admin-i18n-coverage.test.mjs`
  - PASS
- `node tests/admin-commercial-module.test.mjs`
  - PASS
- `node tests/admin-i18n-pricing-module-surface.test.mjs`
  - PASS
- `node scripts/check-rust-verification-matrix.test.mjs`
  - PASS
- `node scripts/rust-verification-workflow.test.mjs`
  - PASS
- `node scripts/check-rust-verification-matrix.mjs --group interface-openapi`
  - PASS
- `node scripts/check-rust-verification-matrix.mjs --group gateway-service`
  - PASS
- `node scripts/check-rust-verification-matrix.mjs --group admin-service`
  - PASS
- `node scripts/check-rust-verification-matrix.mjs --group portal-service`
  - PASS
- `node scripts/check-rust-verification-matrix.mjs --group product-runtime`
  - PASS
- `pnpm.cmd exec tsc -p tsconfig.json --noEmit`
  - PASS
- `cargo test -p sdkwork-api-interface-http gateway_chat_completions_preflight_ignores_invalid_origin_entries -- --exact`
  - PASS
- `cargo test -p sdkwork-api-interface-http gateway_chat_completions_preflight_uses_configured_origin_allowlist -- --exact`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http file_content_route_returns_not_found_error_for_unknown_file -- --exact`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http thread_run_step_route_returns_not_found_error_for_unknown_step -- --exact`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test runs_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test files_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test files_route invalid_request -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http --test files_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test completions_route missing_model -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test completions_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test runs_route thread_and_run_route_returns_invalid_request_for_missing_assistant_id -- --exact --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test runs_route stateful_thread_and_run_route_returns_invalid_request_without_usage -- --exact --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test runs_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test embeddings_route embeddings_route_returns_invalid_request_for_missing_model -- --exact --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test embeddings_route stateful_embeddings_route_returns_invalid_request_for_missing_model_without_usage -- --exact --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test canonical_account_admission stateful_invalid_embeddings_route_releases_platform_credit_hold -- --exact --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test embeddings_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test canonical_account_admission`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test moderations_route moderations_route_returns_invalid_request_for_missing_model -- --exact --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test moderations_route stateful_moderations_route_returns_invalid_request_for_missing_model_without_usage -- --exact --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test moderations_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test images_route images_generation_route_returns_invalid_request_for_missing_model -- --exact --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test images_route stateful_images_generation_route_returns_invalid_request_for_missing_model_without_usage -- --exact --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test images_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test vector_store_files_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test vector_store_files_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test vector_store_file_batches_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test vector_store_file_batches_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test models_route unknown_model -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test models_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test batches_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test batches_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test vector_stores_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test vector_stores_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test assistants_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test assistants_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test evals_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test evals_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test containers_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test containers_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test music_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test music_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test videos_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test videos_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test uploads_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test uploads_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test fine_tuning_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test fine_tuning_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test runs_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http container_file_content_route_returns_not_found_error_for_unknown_file -- --exact`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http music_content_route_returns_not_found_error_for_unknown_track -- --exact`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http video_content_route_returns_not_found_error_for_unknown_video -- --exact`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http --test containers_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http --test music_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http --test videos_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http response_compact_route_returns_invalid_request_for_missing_model -- --exact`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http response_compact_route_returns_ok -- --exact`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http --test responses_route missing_model -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-app-gateway --test responses_api`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-p0-commerce' cargo test -j 1 -p sdkwork-api-app-commerce --test marketing_checkout_closure -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-p0-commerce' cargo test -j 1 -p sdkwork-api-storage-postgres --test integration_postgres --no-run`
  - PASS
- `cargo fmt -p sdkwork-api-app-commerce -p sdkwork-api-interface-portal`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-p0-commerce-account' cargo test -j 1 -p sdkwork-api-interface-portal --test portal_commerce -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-p0-commerce-account' cargo test -j 1 -p sdkwork-api-app-commerce --test marketing_checkout_closure -- --nocapture`
  - PASS
- `cargo fmt -p sdkwork-api-storage-core -p sdkwork-api-storage-sqlite -p sdkwork-api-storage-postgres -p sdkwork-api-app-commerce`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-p0-coupon-comp' cargo test -j 1 -p sdkwork-api-app-commerce --test marketing_checkout_closure -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-p0-coupon-comp' cargo test -j 1 -p sdkwork-api-interface-portal --test portal_commerce -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-p0-coupon-comp' cargo check -p sdkwork-api-storage-postgres`
  - PASS
- `cargo fmt -p sdkwork-api-app-commerce -p sdkwork-api-interface-portal`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-p1-coupon-reclaim' cargo test -j 1 -p sdkwork-api-app-commerce --test marketing_checkout_closure -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-p1-coupon-reclaim' cargo test -j 1 -p sdkwork-api-interface-portal --test marketing_coupon_routes -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http --test responses_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http --test chat_route missing_model -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test chat_route chat_messages_route_returns_not_found_for_unknown_completion -- --exact`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test chat_route stateful_chat_messages_route_returns_not_found_without_usage -- --exact`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test chat_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test conversations_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http --test conversations_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test threads_route not_found -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http --test threads_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http --test chat_stream_route missing_model -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http --test anthropic_messages_route missing_model -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test anthropic_messages_route invalid_request_for_missing_model -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http --test chat_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http --test conversations_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http --test chat_stream_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http --test anthropic_messages_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test gemini_generate_content_route invalid_request_for_missing_model -- --nocapture`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test gemini_generate_content_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-http --test openapi_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-admin --test openapi_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -j 1 -p sdkwork-api-interface-portal --test openapi_route`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http response_retrieve_route_returns_not_found_for_unknown_response -- --exact`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http response_input_items_route_returns_not_found_for_unknown_response -- --exact`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http response_delete_route_returns_not_found_for_unknown_response -- --exact`
  - PASS
- `RUSTFLAGS='-C debuginfo=0' CARGO_TARGET_DIR='target/codex-review-rust' cargo test -p sdkwork-api-interface-http response_cancel_route_returns_not_found_for_unknown_response -- --exact`
  - PASS

### Historical monolithic verification gap

- `cargo check -p gateway-service -p admin-api-service -p portal-api-service -p sdkwork-api-product-runtime`
  - TIMED OUT twice in this environment while compiling native dependencies, most visibly `libz-ng-sys`
  - this remains evidence that the monolithic gate is a poor fit for this workstation
  - it is no longer the only available verification path, because the split package-group matrix now has local passing evidence plus repository-owned automation

## Current Readiness Assessment

The application is not yet commercial-ready.

Reasons already proven in this round:

1. Gateway request handlers still contain panic-on-error paths in real request code.
2. Legacy default router constructors still retain env-parsing panic wrappers, even though explicit `try_*_router()` builders now exist for gateway/admin/portal startup paths.
3. The commercial `zh-CN` catalog now passes structurally, but a large share of newly backfilled commercial strings still render English copy and require module-scoped Chinese translations before the operator experience is localization-complete.
4. Split package-group compile confidence is now green locally, but hosted CI execution and non-Windows runtime proof are still outstanding.

## Findings

### P0-001: Router bootstrap still has panic-prone public env path

**Evidence**

- `crates/sdkwork-api-interface-http/src/lib.rs:2614`
- `crates/sdkwork-api-interface-admin/src/lib.rs:2136`
- `crates/sdkwork-api-interface-portal/src/lib.rs:1665`

All three still contain:

- `try_*_router(...).expect("http exposure config should load from process env")`

This means any direct consumer of the legacy default router constructors can still crash on malformed env-derived exposure config instead of receiving a recoverable startup error. The crash surface is now narrower because explicit fallible builders exist, but the compatibility wrappers still panic.

**Impact**

- brittle startup behavior
- harder failure recovery in embedded/library use
- non-deterministic operator experience when configuration is malformed

**What was fixed now**

- service entrypoints and product runtime no longer need to rely on the env-reparse path
- new injected exposure-config entrypoints were added for gateway/admin/portal router construction
- gateway now exposes `try_gateway_router()`, `try_gateway_router_with_stateless_config(...)`, and `try_gateway_router_with_state(...)`
- admin now exposes `try_admin_router()` and `try_admin_router_with_state(...)`
- portal now exposes `try_portal_router()` and `try_portal_router_with_state(...)`
- targeted regression tests now verify malformed `SDKWORK_BROWSER_ALLOWED_ORIGINS` returns `Err` from the new fallible builders instead of forcing a panic-only construction path

**Remaining work**

- migrate direct callers from the legacy panic wrappers to the new fallible builders in embedded/library code paths
- decide whether the compatibility wrappers should be deprecated after callers move over

### P0-002: Gateway CORS layer previously crashed on invalid allowlist entries

**Evidence**

- previous code path at `crates/sdkwork-api-interface-http/src/lib.rs:1903-1910`
- new regression test in `crates/sdkwork-api-interface-http/tests/cors_preflight.rs`

Before this round, the gateway router built origins with:

- `HeaderValue::from_str(origin).expect(...)`

An invalid configured origin could crash gateway startup or test router construction.

**Impact**

- availability loss during startup
- brittle config handling for browser/admin deployments

**Status**

- fixed in this round by dropping invalid entries instead of panicking
- targeted regression test is green

### P1-001: Admin i18n structural gate is green, but commercial `zh-CN` copy quality is still incomplete

**Evidence**

- `node tests/admin-i18n-coverage.test.mjs`
  - PASS
- `apps/sdkwork-router-admin/tests/admin-i18n-pricing-regressions.test.mjs`
  - PASS
- `apps/sdkwork-router-admin/tests/admin-i18n-marketing-regressions.test.mjs`
  - PASS
- `apps/sdkwork-router-admin/tests/admin-i18n-billing-settlement-regressions.test.mjs`
  - PASS
- `apps/sdkwork-router-admin/tests/admin-i18n-commercial-account-regressions.test.mjs`
  - PASS
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18n.tsx`
  - now contains an explicit `ADMIN_ZH_COMMERCIAL_BACKFILL_TRANSLATIONS` map for the previously missing commercial/admin keys plus dedicated override slices for marketing/coupon, billing/settlement/payment/refund, pricing governance/operator copy, routing/access policy-group copy, and commercial account/admission operator copy

The gate is now structurally green because every used translation key is present. The remaining issue is semantic quality: the generated commercial backfill map currently mirrors English strings for many keys instead of finalized Simplified Chinese operator copy.

**Impact**

- admin quality gates no longer block merges on missing keys
- commercial/admin operator UX is still not localization-complete in `zh-CN`
- translation debt is now explicit and bounded instead of silently growing

**What was fixed now**

- commercial metadata keys were added
- API key group color example placeholder was localized
- rate-limit route/model example placeholders were localized
- pricing placeholders and status labels were localized
- the missing-key backlog was converted into an explicit commercial backfill translation map so `admin-i18n-coverage.test.mjs` is green again
- the coupon/order-audit hotspot now has a dedicated `ADMIN_ZH_MARKETING_TRANSLATIONS` slice covering the loading copy, coupon evidence-chain copy, rollback timeline copy, and coupon-related empty states in the commercial module
- the settlement explorer, settlement ledger, refund timeline, order payment/refund audit, and payment evidence timeline hotspot now has a dedicated `ADMIN_ZH_BILLING_SETTLEMENT_TRANSLATIONS` slice covering its empty states and operator-facing descriptive copy
- the pricing-governance hotspot now has a dedicated `ADMIN_ZH_PRICING_TRANSLATIONS` slice covering the commercial dashboard pricing-governance description, pricing-module operator guidance, gateway pricing-posture guidance, and the pricing-rate prerequisite empty-state prompt
- the pricing module surface now routes charge-unit catalogs, billing-method catalogs, lifecycle summaries, and rounding labels through translated builders backed by `ADMIN_ZH_PRICING_TRANSLATIONS`
- the routing/access hotspot now has a dedicated `ADMIN_ZH_ROUTING_ACCESS_TRANSLATIONS` slice covering API key group policy guidance, routing-profile operator descriptions, bound-group routing posture descriptions, and reusable routing-profile empty states
- the commercial account summary and admission-readiness hotspot now has a dedicated `ADMIN_ZH_COMMERCIAL_ACCOUNT_TRANSLATIONS` slice covering account counts, balances, held-credit posture, and account posture descriptive copy
- the commercial workspace summary and order-audit shell hotspot now has a dedicated `ADMIN_ZH_COMMERCIAL_SURFACE_TRANSLATIONS` slice covering summary facts, order-audit loading/empty-state shell copy, and reconciliation footer wording
- the commercial ledger/refund/order-audit detail hotspot now has a dedicated `ADMIN_ZH_COMMERCIAL_DETAIL_TRANSLATIONS` slice covering table headers, investigation actions, order-detail field labels, evidence fallback labels, and settlement summary amount copy
- the apirouter access/routing/snapshot shell hotspot now has a dedicated `ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS` slice covering API key groups, routing profiles, compiled snapshots, and related workflow/empty-state copy
- the apirouter usage analytics shell now extends `ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS`, covering billing-event analytics cards, routing-evidence summaries, multimodal signal tiles, pricing-posture facts, and recent billing-event shell copy
- the apirouter access-detail and routing-impact hotspot now has a dedicated `ADMIN_ZH_APIROUTER_DETAIL_TRANSLATIONS` slice covering commercial-governance facts, API key group detail labels, routing-impact summaries, and compiled-snapshot detail copy
- the pricing workflow hotspot now extends `ADMIN_ZH_PRICING_TRANSLATIONS`, covering lifecycle-convergence guidance, plan/rate composer field labels, workflow actions, and save-state copy
- the coupon governance/detail hotspot now extends `ADMIN_ZH_MARKETING_TRANSLATIONS`, covering summary cards, lifecycle action labels, compatibility copy, governance-control descriptions, and empty-link fallback labels
- the traffic hotspot now lives in `ADMIN_ZH_TRAFFIC_TRANSLATIONS`, covering billing-event analytics, group/capability/accounting empty states, routing-detail fallback labels, and billing summary guidance
- the apirouter tail now localizes the last UI-facing rate-limit/snapshot labels while explicitly preserving the protocol/example literals `Authorization: Bearer {token}` and `sk-router-live-demo`
- the low-volume pricing-display and settings-shell hotspot is now closed, covering `USD / ...` unit labels, `{count} x {unit}`, and the settings `Shell` group label
- residual effective `zh-CN` inventory now shows 9 English-valued keys still uncovered by dedicated override slices, and all of them are intentional brand/example/protocol literals rather than ordinary UI copy
- the remaining residual literals are `GitHub`, `Google`, `name@example.com`, `Authorization: Bearer {token}`, `sk-router-live-demo`, `{tenant} / {provider}`, `Ctrl K`, and `4102444800000`
- the coupon hotspot is reduced to zero effective semantic-English hits in the current inventory
- the traffic hotspot is reduced to zero effective semantic-English hits in the current inventory
- the pricing hotspot is reduced to zero effective semantic-English hits in the current inventory

**Remaining work**

- keep the remaining brand/example/protocol literals as an explicit policy choice unless product requirements later call for localized variants
- return to the broader requested review areas: payment/order/refund/account-history transactional closure, coupon concurrency/rollback, and traffic-control/failure-recovery/monitoring completeness
- keep `admin-i18n-coverage.test.mjs` green before accepting new admin UI work

### P1-002: Gateway request handlers still contain panic-on-error code in real request paths

**Evidence**

- remaining live request-path samples:
  - `crates/sdkwork-api-interface-http/src/lib.rs:3453`
  - `.expect("models response")` inside stateless model-list fallback serialization
  - `crates/sdkwork-api-interface-http/src/lib.rs:3631`
  - `.expect("chat completions")` inside stateless chat-completion list fallback serialization
  - `crates/sdkwork-api-interface-http/src/lib.rs:3756`
  - `.expect("conversation")` inside stateless conversation creation fallback serialization
  - `crates/sdkwork-api-interface-http/src/lib.rs:3772`
  - `.expect("conversation list")` inside stateless conversation-list fallback serialization
  - `crates/sdkwork-api-interface-http/src/lib.rs:3966`
  - `create_thread(...).expect("thread")` inside stateless thread creation fallback
  - `crates/sdkwork-api-interface-http/src/lib.rs:9833`
  - `.expect("chat completions")` inside stateful chat-completion list fallback serialization
  - `crates/sdkwork-api-interface-http/src/lib.rs:10185`
  - `.expect("conversation")` inside stateful conversation creation fallback serialization
  - `crates/sdkwork-api-interface-http/src/lib.rs:10271`
  - `.expect("conversation list")` inside stateful conversation-list fallback serialization
  - `crates/sdkwork-api-interface-http/src/lib.rs:10869`
  - `create_thread(...).expect("thread")` inside stateful thread creation fallback
- the broader inventory from:
  - `rg -n "\\.(unwrap|expect)\\(" crates/sdkwork-api-interface-http/src crates/sdkwork-api-interface-admin/src crates/sdkwork-api-interface-portal/src`

Sampling confirms this is not limited to test code. Some `expect(...)` calls are inside live request handlers and can convert domain/storage/provider errors into process aborts.

**Impact**

- request-level faults can become availability faults
- failures are not observable through typed HTTP error responses
- retry and auto-failover logic above the service becomes less reliable

**Current required fix direction**

- keep the package-group matrix as the required gate instead of returning to the monolithic `cargo check`
- run the new GitHub workflow and record the first hosted execution result
- extend verification beyond local Windows package compilation into Linux/macOS runtime or packaging evidence

**Historical bootstrap direction already completed**

- replace `expect(...)` in request paths with typed error mapping
- standardize gateway/admin/portal error envelopes
- add regression tests for failure-mode HTTP responses

**What was fixed now**

- unknown local file-content requests now return a controlled 404 envelope
- unknown thread run-step retrievals now return a controlled 404 envelope
- unknown local container-file, music, and video content requests now return controlled 404 envelopes
- local response compaction now validates `model` and returns a controlled 400 invalid-request envelope for missing values
- unknown local response retrieve/input-items/delete/cancel paths now return controlled 404 envelopes in both stateless and stateful router fallbacks
- local `responses` create and `input_tokens` paths now validate `model` and return controlled 400 invalid-request envelopes in both stateless and stateful router fallbacks
- local `responses` stream fallback now validates `model` before emitting SSE frames
- stateful local `responses` and `input_tokens` fallbacks now validate before usage capture so invalid requests do not create billing/usage records
- local OpenAI chat-completion create paths now validate `model` and return controlled 400 invalid-request envelopes in both stateless and stateful router fallbacks
- local OpenAI chat-completion stream fallback now validates `model` before emitting SSE frames
- stateful local OpenAI chat-completion fallbacks now validate before usage capture so invalid requests do not create billing/usage records
- unknown local OpenAI chat-completion retrieve/update/delete paths now return controlled 404 envelopes in both stateless and stateful router fallbacks
- stateful local OpenAI chat-completion retrieve/update/delete fallbacks now defer usage capture until the local lookup succeeds, so missing completions do not create usage records
- unknown local OpenAI chat-completion message-list paths now return controlled 404 envelopes in both stateless and stateful router fallbacks
- stateful local OpenAI chat-completion message-list fallbacks now defer usage capture until the local lookup succeeds, so missing completions do not create usage records
- unknown local conversation retrieve/update/delete paths now return controlled 404 envelopes in both stateless and stateful router fallbacks
- stateful local conversation retrieve/update/delete fallbacks now defer usage capture until the local lookup succeeds, so missing conversations do not create usage records
- unknown local conversation item create/list paths now return controlled 404 envelopes when the parent conversation is missing
- unknown local conversation item retrieve/delete paths now return controlled 404 envelopes when the local item is missing
- stateful local conversation item fallbacks now defer usage capture until the local lookup succeeds, so missing parents/items do not create usage records
- unknown local thread retrieve/update/delete paths now return controlled 404 envelopes in both stateless and stateful router fallbacks
- stateful local thread retrieve/update/delete fallbacks now defer usage capture until the local lookup succeeds, so missing threads do not create usage records
- unknown local thread message create/list paths now return controlled 404 envelopes when the parent thread is missing
- unknown local thread message retrieve/update/delete paths now return controlled 404 envelopes when the local message is missing
- stateful local thread message fallbacks now defer usage capture until the local thread/thread-message lookup succeeds, so missing threads/messages do not create usage records
- unknown local thread run create/list paths now return controlled 404 envelopes when the parent thread is missing
- unknown local thread run retrieve/update/cancel/submit-tool-outputs/steps-list paths now return controlled 404 envelopes when the local run is missing
- stateful local thread run and run-step fallbacks now defer usage capture until the local thread/run/step lookup succeeds, so missing threads/runs/steps do not create usage records
- unknown local file retrieve/delete paths now return controlled 404 envelopes in both stateless and stateful router fallbacks
- stateful local file retrieve/delete fallbacks now defer usage capture until the local file lookup succeeds, so missing files do not create usage records
- unknown local vector-store file retrieve/delete paths now return controlled 404 envelopes in both stateless and stateful router fallbacks
- stateful local vector-store file retrieve/delete fallbacks now defer usage capture until the local vector-store file lookup succeeds, so missing vector-store files do not create usage records
- Anthropic `count_tokens` fallback now maps local invalid-model errors to an Anthropic error envelope instead of panicking
- Anthropic `messages` JSON and stream fallbacks now validate `model` and map local invalid-model errors to Anthropic invalid-request envelopes instead of panicking or emitting invalid SSE
- stateful Anthropic local `messages` fallbacks now validate before usage capture so invalid requests do not create billing/usage records
- Gemini `generateContent`, `streamGenerateContent`, and `countTokens` fallbacks now validate `model` and map local invalid-model errors to Gemini invalid-request envelopes instead of panicking or returning synthetic success responses
- stateful Gemini local fallback paths now validate before usage capture so invalid requests do not create billing/usage records
- unknown local eval retrieve/update/delete paths now return controlled 404 envelopes instead of incorrectly returning synthetic success payloads
- unknown local eval-run list/create paths now return controlled 404 envelopes when the parent eval is missing
- unknown local eval-run retrieve/delete/cancel and output-items list paths now return controlled 404 envelopes when the local run is missing
- unknown local eval-run output-item retrieve paths now return controlled 404 envelopes when the local output item is missing
- stateful local eval and eval-run fallbacks now defer usage capture until the local eval/run/output-item lookup succeeds, so missing resources do not create usage records
- unknown local container retrieve/delete paths now return controlled 404 envelopes instead of incorrectly returning synthetic success payloads
- unknown local container-file create/list paths now return controlled 404 envelopes when the parent container is missing
- unknown local container-file retrieve/delete/content paths now return controlled 404 envelopes when the local file is missing
- stateful local container and container-file fallbacks now defer usage capture until the local container/file lookup succeeds, so missing resources do not create usage records
- unknown local music retrieve/delete paths now return controlled 404 envelopes instead of incorrectly returning synthetic success payloads
- stateful local music retrieve/delete/content fallbacks now defer usage capture until the local music lookup succeeds, so missing tracks do not create usage records
- unknown local video retrieve/delete paths now return controlled 404 envelopes instead of incorrectly returning synthetic success payloads
- stateful local video retrieve/delete/content fallbacks now defer usage capture until the local video lookup succeeds, so missing videos do not create usage records
- missing local upload sessions now return controlled 404 envelopes for upload-part create, upload complete, and upload cancel fallbacks instead of incorrectly returning synthetic success payloads
- stateful local upload-part/complete/cancel fallbacks now defer usage capture until the local upload lookup succeeds, so missing upload sessions do not create usage records
- missing local fine-tuning jobs now return controlled 404 envelopes for retrieve/cancel/events/checkpoints/pause/resume fallbacks instead of incorrectly returning synthetic success payloads
- missing local fine-tuning checkpoints now return controlled 404 envelopes for checkpoint-permission create/list fallbacks, and missing local checkpoint permissions now return controlled 404 envelopes for permission delete fallbacks
- stateful local fine-tuning job/checkpoint/permission fallbacks now defer usage capture until the local lookup succeeds, so missing resources do not create usage records
- local file upload create fallbacks now validate `purpose` and `filename` before serializing the local response, so blank values return controlled 400 invalid-request envelopes instead of panicking
- stateful local file upload create fallbacks now validate before usage capture, so invalid local upload requests do not create billing/usage records
- local completion create fallbacks now validate `model` before serializing the local response, so blank values return controlled 400 invalid-request envelopes instead of panicking
- stateful local completion create fallbacks now validate before usage capture, so invalid local completion requests do not create billing/usage records
- local thread-and-run create fallbacks now validate `assistant_id` before serializing the local response, so blank values return controlled 400 invalid-request envelopes instead of panicking
- stateful local thread-and-run create fallbacks now validate before usage capture, so invalid local thread-and-run requests do not create billing/usage records
- local embeddings create fallbacks now validate `model` before serializing the local response, so blank values return controlled 400 invalid-request envelopes instead of panicking
- stateful local embeddings create fallbacks now validate before usage capture, so invalid local embedding requests do not create billing/usage records
- stateful local embeddings create fallbacks now release canonical commercial billing holds before returning invalid local-request responses, so invalid local input no longer leaves held balance behind
- local moderations create fallbacks now validate `model` before serializing the local response, so blank values return controlled 400 invalid-request envelopes instead of panicking
- stateful local moderations create fallbacks now validate before usage capture, so invalid local moderation requests do not create billing/usage records
- local image-generation create fallbacks now validate `model` before serializing the local response, so blank values return controlled 400 invalid-request envelopes instead of panicking
- stateful local image-generation create fallbacks now validate before usage recording, so invalid local image-generation requests do not create billing/usage records
- re-review of the remaining sampled list/create `expect(...)` sites found no currently proven business failure surface in the fixed-structure local fallbacks for model list, chat-completion list, conversation create/list, or thread create; image edit/variation also no longer qualify as the next invalid-model slice because their contracts intentionally allow `model: Option<String>` with a default
- targeted regression tests were added and verified for each completed failure-mode slice

### P2-001: Cross-package build confidence improved, but CI/runtime breadth remains incomplete

**Evidence**

- `cargo check -p gateway-service -p admin-api-service -p portal-api-service -p sdkwork-api-product-runtime`
  - timed out locally during native dependency compilation
- `cargo check -p sdkwork-api-product-runtime --lib`
  - also timed out locally during `libz-ng-sys`
- `node scripts/check-rust-verification-matrix.test.mjs`
  - PASS
- `node scripts/rust-verification-workflow.test.mjs`
  - PASS
- `node scripts/check-rust-verification-matrix.mjs --group interface-openapi`
  - PASS
- `node scripts/check-rust-verification-matrix.mjs --group gateway-service`
  - PASS
- `node scripts/check-rust-verification-matrix.mjs --group admin-service`
  - PASS
- `node scripts/check-rust-verification-matrix.mjs --group portal-service`
  - PASS
- `node scripts/check-rust-verification-matrix.mjs --group product-runtime`
  - PASS
- `node --test --experimental-test-isolation=none scripts/dev/tests/windows-rust-toolchain-guard.test.mjs`
  - PASS
- `node --test --experimental-test-isolation=none scripts/rust-verification-workflow.test.mjs`
  - PASS
- `node scripts/check-rust-verification-matrix.mjs --group workspace`
  - PASS
- `CARGO_TARGET_DIR=t3 cargo check --workspace -j 1`
  - PASS on the Windows workstation once the build used a short target directory instead of the default long shared `target` path

**Impact**

- local service/runtime package-group compile confidence is now materially better than it was at the start of the review
- the repository-owned verification entrypoint can now also prove a full Windows workspace build when it uses the managed short target-dir strategy
- the hosted Rust verification workflow now exposes a manual `windows-latest` `workspace` lane, so future CI evidence can be collected without slowing the default PR split-package matrix
- cross-platform release readiness still cannot be claimed from fresh hosted CI or non-Windows runtime evidence

**Current required fix direction**

- keep the package-group matrix as the required default gate and use the new `workspace` group as the local deep-validation path
- keep Windows verification on the managed short target-dir flow; do not reintroduce `RUSTFLAGS='-C debuginfo=0'` as a blanket workaround
- run the new GitHub workflow and record the first hosted execution result
- extend verification beyond local Windows package compilation into Linux/macOS runtime or packaging evidence

**Historical bootstrap direction already completed**

- add CI/native dependency caching guidance for this workspace
- split verification into smaller package gates that still prove the changed surfaces
- add documented “required green matrix” for gateway/admin/portal/product runtime

## Recommended Next Slice

1. P1: resume the broader commercial-system review with payment/order/refund/account-history transactional closure and failure-mode analysis.
2. P1: review coupon concurrency, rollback, and inventory restoration under failure and retry conditions.
3. P1: review traffic control, failure recovery, monitoring, and auto-failover completeness.
4. P3: track the remaining fixed-structure request-path `expect(...)` sites as hygiene debt unless a real domain failure surface is demonstrated in later review.

## Pending Review Areas Not Yet Closed

The following areas were requested by the user but are not yet fully closed by evidence in this round:

- full payment/checkout/refund/account-history transaction review
- coupon concurrency and inventory rollback under failure
- high-load performance and traffic control behavior
- monitoring, auto-failover, and failure recovery instrumentation completeness
- cross-platform packaging/runtime verification on non-Windows hosts

These remain in scope for subsequent review/fix cycles.
