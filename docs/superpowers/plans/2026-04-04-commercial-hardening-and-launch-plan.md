# Commercial Hardening And Launch Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** close the remaining security, billing-correctness, resilience, and observability gaps that currently block `sdkwork-api-router` from a real commercial launch.

**Architecture:** keep the plugin-first router and canonical account direction, but harden the commercial control path in the correct order: remove unsafe settlement shortcuts and plaintext secret retention first, then add provider execution resilience, then finish canonical multimodal settlement and real payment rails. Do not widen product scope again until payment, auth, routing failover, and monitoring are production-safe.

**Tech Stack:** Rust, Axum, sqlx, SQLite, PostgreSQL, reqwest, React package apps, cargo test, pnpm

---

### Task 1: Remove unsafe portal-side payment settlement controls

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `crates/sdkwork-api-app-commerce/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_commerce.rs`
- Modify: `docs/api-reference/portal-api.md`
- Modify: `docs/getting-started/public-portal.md`

- [x] **Step 1: Add failing portal route tests proving workspace users cannot self-settle paid orders**
- [x] **Step 2: Add failing portal route tests proving `/portal/commerce/orders/{order_id}/payment-events` rejects direct end-user settlement events**
- [x] **Step 3: Restrict portal settlement actions to zero-pay or lab-only flows guarded by explicit environment flags**
- [x] **Step 4: Introduce a separate provider-callback ingestion seam that is not callable by normal portal JWT sessions**
- [x] **Step 5: Run `cargo test -p sdkwork-api-interface-portal --test portal_commerce -- --nocapture`**

### Task 2: Eliminate plaintext API key retention and insecure bootstrap defaults

**Files:**
- Modify: `crates/sdkwork-api-domain-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-config/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `docs/api-reference/admin-api.md`
- Modify: `docs/operations/configuration.md`

- [x] **Step 1: Add failing identity tests proving persisted gateway API keys never retain `raw_key` after create**
- [x] **Step 2: Add failing admin API tests proving list or detail endpoints never serialize plaintext API keys after creation**
- [x] **Step 3: Change gateway API key persistence to store only hash, prefix, creation metadata, and one-time reveal payloads**
- [x] **Step 4: Add startup validation that rejects local-dev JWT secrets, master keys, and default passwords outside explicit dev mode**
- [x] **Step 5: Run `cargo test -p sdkwork-api-app-identity -- --nocapture` and `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -- --nocapture`**

### Task 3: Add provider execution timeouts, retries, circuit breaking, and request failover

**Files:**
- Modify: `crates/sdkwork-api-provider-openai/src/lib.rs`
- Modify: `crates/sdkwork-api-provider-openrouter/src/lib.rs`
- Modify: `crates/sdkwork-api-provider-ollama/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-routing/src/lib.rs`
- Create: `crates/sdkwork-api-app-gateway/tests/provider_failover.rs`
- Create: `crates/sdkwork-api-provider-openai/tests/http_timeouts.rs`

- [x] **Step 1: Add failing provider tests for request timeout, connect timeout, and retryable transport errors**
- [x] **Step 2: Build shared provider client construction with explicit connect, request, pool, and keepalive settings**
- [x] **Step 3: Add route-attempt orchestration that can retry the selected provider and fail over to the next ranked candidate on retryable failures**
- [x] **Step 4: Persist provider-attempt outcome evidence and short-lived circuit state so unhealthy providers are temporarily suppressed**
- [x] **Step 5: Run `cargo test -p sdkwork-api-app-gateway --test provider_failover -- --nocapture` and `cargo test -p sdkwork-api-provider-openai --test http_timeouts -- --nocapture`**

### Task 4: Move flow control from basic fixed-window request limits to layered commercial admission

**Files:**
- Modify: `crates/sdkwork-api-app-rate-limit/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Create: `crates/sdkwork-api-interface-http/tests/traffic_control.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`

- [x] **Step 1: Add failing tests for per-project concurrency caps, per-api-key concurrency caps, and provider-level backpressure**
- [x] **Step 2: Add explicit commercial admission policies for requests, concurrency, burst, token budget, and media-job queue depth**
- [x] **Step 3: Back the hot-path flow-control state with a cache-friendly counter path instead of policy scan plus SQL mutation only**
- [x] **Step 4: Expose admin and portal read models for live pressure, throttled keys, and saturated providers**
- [x] **Step 5: Run `cargo test -p sdkwork-api-interface-http --test traffic_control -- --nocapture`**

### Task 5: Finish canonical settlement for streaming and multimodal routes

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-usage/src/lib.rs`
- Create: `crates/sdkwork-api-interface-http/tests/multimodal_settlement.rs`

Progress note:
- fixed request-priced media routes now use canonical hold/capture/release for `/v1/images/generations`, `/v1/audio/transcriptions`, `/v1/audio/translations`, and `/v1/audio/speech`.
- `responses` SSE streams now admit canonical account holds, settle on `response.completed`, and release held balance when the upstream stream errors or finishes without a completion event.
- `music` create now admits a buffered canonical hold, stores the upstream `music_id` on the request meter fact, defers permanent billing while the job is pending, and reconciles exactly once on retrieve or list when a terminal duration becomes available.
- admin pricing now exposes `per_second_music`, and the billing core can settle delayed music jobs with `UsageCaptureStatus::Reconciled` instead of reusing coarse request pricing.
- `videos` create now admits a canonical hold when the model price uses duration-aware units, stores the created `video_id` on the canonical request meter fact, and reconciles exactly once on retrieve or list when terminal `duration_seconds` becomes available.
- admin-compatible video pricing now recognizes `per_minute_video` and `per_second_video` style units in canonical settlement, while local video stubs expose terminal status and duration so fallback paths do not strand held balance.
- long-running video mutation routes (`remix`, `extend`, `edits`, `extensions`) now admit canonical holds, annotate the created child `video_id` onto the request meter fact, and reconcile exactly once on later retrieve with duration-aware settlement instead of fixed compatibility charging.
- mutation routes can now resolve canonical video pricing even when the request has no explicit `model` by selecting the unique active video-priced model for the routed provider and channel; this preserves canonical admission on edit-style APIs without inventing route-local pseudo-model IDs.
- verification now includes `multimodal_settlement.rs` for the mutation-path regressions, while the broader stream, image, audio, music, and video canonical settlement coverage remains in `canonical_account_admission.rs`.

- [x] **Step 1: Add failing tests for stream-final settlement, image settlement, audio settlement, music settlement, and video settlement**
- [x] **Step 2: Replace fixed compatibility charges on media routes with canonical hold, capture, release, and reconciliation inputs**
- [x] **Step 3: Add stream-final usage capture hooks so SSE and long-lived responses do not stay on coarse fixed charging**
- [x] **Step 4: Add callback or polling reconciliation records for long-running media requests**
- [x] **Step 5: Run `cargo test -p sdkwork-api-interface-http --test multimodal_settlement -- --nocapture`**

### Task 6: Add real payment adapters and finance-grade callback handling

**Files:**
- Create: `crates/sdkwork-api-domain-payment/src/lib.rs`
- Create: `crates/sdkwork-api-app-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-app-commerce/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Create: `crates/sdkwork-api-interface-portal/tests/payment_provider_callbacks.rs`
- Modify: `docs/api-reference/portal-api.md`
- Modify: `docs/getting-started/public-portal.md`

Progress note:
- Stripe checkout, webhook verification, Alipay callback ingestion, WeChat callback ingestion, duplicate-event suppression, and payment evidence persistence are now in place.
- Payment settlement now issues canonical portal-workspace identity records and bindings, creates or reuses a primary canonical account, and writes recharge grant lots plus grant ledger and allocation evidence for recharge orders.
- Recharge fulfillment no longer mutates quota policy limits for `recharge_pack` or `custom_recharge`; both portal and admin billing summaries now prefer canonical account balance and keep legacy quota values as explicit compatibility fields.
- Remaining open scope in Task 6 is deciding whether workspace commerce should continue using a user-scoped canonical wallet or move to an explicit shared workspace billing account, plus deeper finance-grade reconciliation and reporting views.

- [x] **Step 1: Add failing tests for Stripe checkout session creation and signed webhook verification**
- [x] **Step 2: Add failing tests for Alipay and WeChat provider callback ingestion and duplicate-event suppression**
- [x] **Step 3: Introduce canonical payment-order, payment-attempt, webhook-event, refund, and dispute records**
- [x] **Step 4: Route payment success into canonical account benefit lots and finance evidence instead of compatibility quota mutation**
- [x] **Step 5: Run `cargo test -p sdkwork-api-interface-portal --test payment_provider_callbacks -- --nocapture`**

### Task 7: Upgrade observability from basic HTTP counters to commercial-grade telemetry

**Files:**
- Modify: `crates/sdkwork-api-observability/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Create: `docs/operations/alerts-and-slos.md`

Progress note:
- HTTP telemetry now carries route, tenant, model, provider, billing mode, retry, failover, and payment outcome labels with in-process cardinality caps.
- Request and provider latency are exported as histogram-style metrics instead of only raw sum/count aggregates.
- Gateway failover, payment callback replay, settlement replay, throttling, and canonical hold capture/release failures now emit structured commercial events.
- Portal payment callbacks now emit explicit callback outcome metrics, and provider failover paths emit provider execution counters plus failover activation events.
- Operator alert rules and dashboard guidance are documented in `docs/operations/alerts-and-slos.md`.

- [x] **Step 1: Add route, provider, model, tenant, billing, retry, failover, and payment outcome metric dimensions with cardinality controls**
- [x] **Step 2: Replace duration sum or count only metrics with histogram-style latency buckets for gateway and provider execution**
- [x] **Step 3: Emit structured events for hold failure, settlement replay, failover activation, callback replay, and throttling**
- [x] **Step 4: Publish operator alert rules for payment callback lag, provider error rate, rate-limit saturation, and settlement mismatch**
- [x] **Step 5: Run `cargo check -p sdkwork-api-observability -p sdkwork-api-interface-http -p sdkwork-api-interface-admin -p sdkwork-api-interface-portal`**

### Task 8: Add enterprise identity and control-plane governance

**Files:**
- Modify: `crates/sdkwork-api-domain-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Create: `crates/sdkwork-api-interface-admin/tests/rbac_identity.rs`
- Create: `crates/sdkwork-api-interface-portal/tests/workspace_membership.rs`

Progress note:
- admin RBAC now recognizes `super_admin`, `platform_operator`, `finance_operator`, and `read_only_operator`, and route authorization separates platform reads or writes from finance reads or writes plus super-admin-only secrets and identity management.
- portal identity now persists workspace membership records, lists all memberships for a user, rejects workspace switches to non-member targets, and reissues scoped sessions against the selected tenant or project pair.
- admin audit events are now durably persisted in `ai_admin_audit_events` with actor id, email, role, action, resource, approval scope, and optional tenant, project, and provider targeting metadata.
- high-risk control-plane mutations now emit durable audit evidence for:
  - model price create and delete
  - upstream credential create and delete
  - admin user create, update, status change, password reset, and delete
  - portal user create, update, status change, password reset, and delete
  - API key group create, update, status change, and delete
  - gateway API key create, update, status change, and delete
- pricing writes now require `finance_operator` or `super_admin`, while secrets and identity mutations remain `super_admin` only. The current codebase still has no direct admin-side payment mutation routes, so the new audit substrate is the approval boundary for future finance write surfaces rather than a second legacy path.

- [x] **Step 1: Add failing tests for admin RBAC, finance-only roles, and read-only operator scopes**
- [x] **Step 2: Add failing tests for portal multi-workspace membership and workspace switching**
- [x] **Step 3: Replace single-workspace portal identity with membership records and scoped claims**
- [x] **Step 4: Add admin roles, permissions, audit actor metadata, and approval boundaries for pricing, payments, and secrets**
- [x] **Step 5: Run `cargo test -p sdkwork-api-interface-admin --test rbac_identity -- --nocapture` and `cargo test -p sdkwork-api-interface-portal --test workspace_membership -- --nocapture`**
