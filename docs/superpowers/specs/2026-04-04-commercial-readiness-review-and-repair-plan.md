# Commercial Readiness Review And Repair Plan

**Status:** checked against the local `feature/marketing-foundation` worktree on 2026-04-04

**Scope:** gateway request path, canonical identity and account kernel cutover, commerce and marketing flows, admin and portal control plane, storage backends, and multimodal commercial completeness

## Executive Verdict

`sdkwork-api-router` now has a stronger architecture foundation than most open-source API routers:

- backend seams are modular instead of monolithic
- routing, billing, quota, storage, cache, and runtime boundaries are explicit
- admin and portal are already package-split React products instead of a single tangled console
- multimodal route surface breadth is already commercially useful

But it is still **not ready for a full paid commercial launch**.

The codebase is now at a strong platform-foundation stage, and parts of the canonical billing cutover are real. Non-streaming `/v1/chat/completions`, `/v1/responses`, `/v1/completions`, and `/v1/embeddings` can already use canonical price-aware admission and settlement. That is meaningful progress.

The blocking issue is that the remaining gaps are concentrated in the exact places that determine whether a router can survive real commercial traffic:

- end-to-end identity cutover
- settlement correctness across every request path
- canonical account and commerce convergence
- real payment provider integration
- admin and portal financial control-plane completeness
- durable multimodal async execution
- production-grade multi-database parity

The right decision is **not** to rewrite the architecture.

The right decision is:

1. keep the current plugin-first architecture
2. finish the canonical identity, account, settlement, and commerce kernels
3. cut admin and portal onto those kernels
4. add the missing async media and payment infrastructure

## Findings

### P0-1. Canonical-only API keys still cannot authenticate the gateway end to end

**Why this matters:** canonical identity cannot become the production source of truth while the gateway still hard-depends on legacy workspace-string request context resolution.

**Evidence:**

- gateway middleware still loads `GatewayRequestContext` through `resolve_gateway_request_context(...)` before accepting the request in [crates/sdkwork-api-interface-http/src/lib.rs](../../../crates/sdkwork-api-interface-http/src/lib.rs#L597)
- request extension injection also still depends on legacy context resolution in [crates/sdkwork-api-interface-http/src/lib.rs](../../../crates/sdkwork-api-interface-http/src/lib.rs#L839)
- the legacy request context remains string-based in [crates/sdkwork-api-app-identity/src/lib.rs](../../../crates/sdkwork-api-app-identity/src/lib.rs#L66)
- the canonical subject is bigint-oriented in [crates/sdkwork-api-domain-identity/src/lib.rs](../../../crates/sdkwork-api-domain-identity/src/lib.rs#L419)

**Impact:** canonical API keys, canonical users, and canonical accounts cannot fully replace the compatibility-era identity path. This blocks a clean commercial kernel.

**Fix direction:** introduce a canonical gateway auth bridge that derives the request execution scope from `GatewayAuthSubject` first, then maps to workspace/project compatibility only where still required.

### P0-2. Settlement correctness is still incomplete for streaming and most multimodal routes

**Why this matters:** the system cannot claim commercial billing correctness while only a subset of synchronous text routes use canonical holds and settlement.

**Evidence:**

- chat canonical admission explicitly skips streaming in [crates/sdkwork-api-interface-http/src/lib.rs](../../../crates/sdkwork-api-interface-http/src/lib.rs#L17622)
- moderation still ends on compatibility-era usage recording in [crates/sdkwork-api-interface-http/src/lib.rs](../../../crates/sdkwork-api-interface-http/src/lib.rs#L10221)
- image generation still records legacy project usage instead of canonical settlement in [crates/sdkwork-api-interface-http/src/lib.rs](../../../crates/sdkwork-api-interface-http/src/lib.rs#L10320)

**Impact:** streamed text, moderation, image, audio, video, music, and compatibility-family routes still have charge leakage and reconciliation risk relative to the canonical account design.

**Fix direction:** extend the canonical admission and capture model from the current non-streaming text routes into:

- `/v1/moderations`
- all synchronous multimodal routes
- stream-finalization settlement for chat, responses, and compatibility stream families
- webhook or callback reconciliation for long-running media work

### P0-3. Commerce still mutates project quota instead of the canonical account kernel

**Why this matters:** commercial checkout and recharge cannot remain tied to compatibility-era quota mutation if the target system is account-based, benefit-lot-aware, and settlement-reproducible.

**Evidence:**

- commerce still applies purchased value by increasing project quota in [crates/sdkwork-api-app-commerce/src/lib.rs](../../../crates/sdkwork-api-app-commerce/src/lib.rs#L949)
- the payable effect is still `granted_units + bonus_units` instead of canonical account lots in [crates/sdkwork-api-app-commerce/src/lib.rs](../../../crates/sdkwork-api-app-commerce/src/lib.rs#L957)

**Impact:** payments, coupons, recharge packs, and subscriptions are not yet aligned with the new canonical account, hold, ledger, and benefit-lot model.

**Fix direction:** cut commerce settlement over to:

- `ai_account`
- `ai_account_benefit_lot`
- `ai_account_ledger_entry`
- `ai_request_settlement`

Compatibility quota mutation should become a temporary projection, not the source of truth.

### P0-4. Payment provider integration is still simulated, not real

**Why this matters:** a commercial router cannot be called payment-ready while checkout is still driven by manual event simulation and planned provider handoff.

**Evidence:**

- checkout methods still expose `provider_handoff` as a planned seam in [crates/sdkwork-api-app-commerce/src/lib.rs](../../../crates/sdkwork-api-app-commerce/src/lib.rs#L1492)
- the portal billing UI is explicitly simulating provider settlement and webhook outcomes in [apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx](../../../apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx#L1901)
- portal docs explicitly say live checkout is not yet included in [docs/getting-started/public-portal.md](../../getting-started/public-portal.md#L71)

**Impact:** Stripe, Alipay, and WeChat Pay are not operational. There is no production-grade webhook verification, idempotent provider callback pipeline, dispute/refund workflow, or finance-grade payment evidence chain.

**Fix direction:** add a real payment adapter layer with:

- provider session creation
- signed webhook verification
- idempotent payment event ingestion
- settlement, cancellation, failure, refund, and dispute state transitions
- finance export evidence

### P0-5. PostgreSQL is still not a real canonical identity and account backend

**Why this matters:** the new commercial kernel cannot be production-complete if only SQLite has real canonical identity and account behavior.

**Evidence:**

- PostgreSQL exposes `account_kernel()` in [crates/sdkwork-api-storage-postgres/src/lib.rs](../../../crates/sdkwork-api-storage-postgres/src/lib.rs#L7523)
- but `AccountKernelStore` is still an empty impl in [crates/sdkwork-api-storage-postgres/src/lib.rs](../../../crates/sdkwork-api-storage-postgres/src/lib.rs#L8345)
- no `identity_kernel()` accessor or `IdentityKernelStore` impl was found in [crates/sdkwork-api-storage-postgres/src/lib.rs](../../../crates/sdkwork-api-storage-postgres/src/lib.rs)

**Impact:** the strongest commercial path is still effectively SQLite-first, which is not the right production posture for a commercial cloud gateway.

**Fix direction:** implement full PostgreSQL parity for:

- canonical identity CRUD
- canonical account CRUD
- hold capture and release orchestration
- pricing plan and rate persistence
- benefit-lot allocation persistence

### P0-6. Admin and portal still operate on legacy billing summaries, not canonical finance read models

**Why this matters:** even if canonical settlement improves inside the gateway, operators and tenants still cannot see the real commercial truth unless the control plane reads canonical balances, holds, allocations, and settlements.

**Evidence:**

- portal billing summary still derives from legacy ledger and quota policy snapshots in [crates/sdkwork-api-interface-portal/src/lib.rs](../../../crates/sdkwork-api-interface-portal/src/lib.rs#L1594)
- admin billing summary still uses the compatibility summary surface in [crates/sdkwork-api-interface-admin/src/lib.rs](../../../crates/sdkwork-api-interface-admin/src/lib.rs#L3055)

**Impact:** operators cannot inspect canonical account balances, active holds, settlement lifecycle, or pricing-plan-specific charge evidence. Portal tenants cannot see account-grade balance and settlement history either.

**Fix direction:** add canonical finance read models and cut:

- admin billing summary
- portal billing summary
- ledger explorer
- settlement explorer
- account runway and recharge posture

onto canonical tables first.

### P1-1. Pricing-plan schema exists, but pricing-plan governance APIs do not

**Why this matters:** advanced commercial systems need versioned tariffs, reproducible pricing history, and finance-safe plan evolution. A schema alone is not enough.

**Evidence:**

- canonical pricing plan and rate tables already exist in SQLite in [crates/sdkwork-api-storage-sqlite/src/lib.rs](../../../crates/sdkwork-api-storage-sqlite/src/lib.rs#L1161)
- admin currently exposes only model-price CRUD, not pricing-plan or pricing-rate governance in [crates/sdkwork-api-interface-admin/src/lib.rs](../../../crates/sdkwork-api-interface-admin/src/lib.rs#L1355)

**Impact:** model unit prices can be managed, but plan-version governance, enterprise pricing overlays, historical tariff replay, and customer pricing packages are not yet control-plane products.

**Fix direction:** add canonical pricing-plan and pricing-rate admin APIs plus portal-readable effective pricing views.

### P1-2. Coupon and marketing are split between a legacy coupon path and a newer canonical marketing kernel

**Why this matters:** the marketing system is moving in the right direction, but the write path is still split-brain.

**Evidence:**

- the coupon app explicitly says it persists the compatibility-era coupon model in [crates/sdkwork-api-app-coupon/src/lib.rs](../../../crates/sdkwork-api-app-coupon/src/lib.rs#L6)
- admin still creates and deletes coupons through `/admin/coupons` in [crates/sdkwork-api-interface-admin/src/lib.rs](../../../crates/sdkwork-api-interface-admin/src/lib.rs#L1239)
- canonical marketing routes in admin are currently read-oriented `GET` surfaces in [crates/sdkwork-api-interface-admin/src/lib.rs](../../../crates/sdkwork-api-interface-admin/src/lib.rs#L1244)

**Impact:** the system now has a better canonical marketing model, but operator write flows still anchor partly in legacy coupon shape, which will create lifecycle drift.

**Fix direction:** promote canonical marketing to the only operator write path:

- coupon template CRUD
- code batch issuance
- campaign lifecycle
- claim and redemption operations
- referral and invite program management

### P1-3. Durable async media job infrastructure is still missing

**Why this matters:** `new-api` still has practical product-density advantages in media operations because advanced commercial media products need durable job state, retries, artifacts, callbacks, and operator visibility.

**Evidence:**

- the current internal audit still records the async job kernel as missing in [docs/superpowers/specs/2026-04-03-commercial-system-gap-assessment-and-target-solution.md](2026-04-03-commercial-system-gap-assessment-and-target-solution.md#L136)
- the target tables are still listed as design targets, not shipped backend subsystems, in [docs/superpowers/specs/2026-04-03-commercial-system-gap-assessment-and-target-solution.md](2026-04-03-commercial-system-gap-assessment-and-target-solution.md#L156)

**Impact:** image, video, audio, and music routes exist, but long-running generation is not yet a durable commercial workload platform.

**Fix direction:** add:

- `ai_async_job`
- `ai_async_job_attempt`
- `ai_generated_asset`
- `ai_provider_callback_event`
- `ai_async_job_webhook_delivery`

then build admin and portal job workbenches on top.

### P1-4. Plugin governance is still runtime-centric instead of platform-wide

**Why this matters:** the current architecture is correct, but advanced commercial systems need a unified inventory of all installed plugins, not only extension runtime installations.

**Evidence:**

- admin already exposes extension installation and runtime rollout endpoints in [crates/sdkwork-api-interface-admin/src/lib.rs](../../../crates/sdkwork-api-interface-admin/src/lib.rs#L1363)
- there is still no equivalent platform inventory for storage drivers, cache drivers, policy plugins, or backend product modules in the same control plane

**Impact:** operators cannot see one authoritative compatibility and health surface for the entire plugin platform.

**Fix direction:** add a platform plugin registry and control-plane inventory across:

- storage drivers
- cache drivers
- policy plugins
- runtime plugins
- backend product modules

### P1-5. MySQL and LibSQL are still dialect stubs, not real backends

**Why this matters:** a commercial platform should fail honestly, but long term it also needs real production backends beyond SQLite.

**Evidence:**

- runtime explicitly fails unsupported dialects in [crates/sdkwork-api-app-runtime/src/lib.rs](../../../crates/sdkwork-api-app-runtime/src/lib.rs#L499)
- MySQL and LibSQL crates only expose dialect identity in [crates/sdkwork-api-storage-mysql/src/lib.rs](../../../crates/sdkwork-api-storage-mysql/src/lib.rs#L1) and [crates/sdkwork-api-storage-libsql/src/lib.rs](../../../crates/sdkwork-api-storage-libsql/src/lib.rs#L1)

**Impact:** the architecture is honest, but the multi-database story is not yet commercially complete.

**Fix direction:** finish PostgreSQL first, then ship real MySQL, then LibSQL.

### P2-1. Public commercial account experience is still incomplete by the project’s own docs

**Why this matters:** the product is correctly honest about its current scope, but that scope is still below advanced commercial SaaS expectations.

**Evidence:**

- the portal docs explicitly state that it does not yet include OAuth or SSO in [docs/getting-started/public-portal.md](../../getting-started/public-portal.md#L71)
- the same docs state that live checkout and payment settlement are also not yet included in [docs/getting-started/public-portal.md](../../getting-started/public-portal.md#L71)

**Impact:** enterprise sign-in, commercial onboarding, and self-serve monetization are still incomplete.

**Fix direction:** add enterprise auth, live commerce, and account operations after the P0 finance kernel is complete.

## Comparison To Advanced Commercial Routers

## Where the current system is already stronger

Compared with the common `new-api` style monolith and the typical open-source router baseline, the current repository is already better in these ways:

- explicit storage, cache, policy, runtime, app, and interface boundaries
- plugin-oriented architecture instead of a transport-first product monolith
- clearer React package boundaries in admin and portal
- richer routing control-plane direction
- stronger long-term extensibility for provider, storage, and policy evolution

This architecture should be preserved.

## Where advanced commercial systems are still ahead

Compared with OpenRouter-class commercial expectations and the denser multimodal product posture visible in `new-api`, the current system still trails in:

- full-route settlement correctness
- real payment rails
- finance-grade account and pricing governance
- durable async media execution
- operator-facing reconciliation and settlement tooling
- platform-wide plugin governance
- production database parity

That is why the correct next step is **closure**, not **rewriting**.

## Best Upgrade Path

The best path is a **canonical-core completion strategy**:

1. keep the current modular architecture
2. treat all compatibility-era quota, coupon, and billing summaries as temporary adapters
3. move gateway, commerce, admin, and portal onto canonical identity, account, pricing, settlement, and marketing kernels
4. add durable async job and payment-provider infrastructure after the finance kernel is stable

The wrong path would be:

- expanding legacy quota mutation further
- adding more payment features on top of simulated settlement
- widening the compatibility coupon model
- rewriting backend or frontend package structure instead of finishing the commercial kernels

## Repair Plan

### P0: Required before paid commercial launch

1. **Canonical gateway auth cutover**
   - Build a bridge from `GatewayAuthSubject` to request execution scope
   - Let canonical API keys authenticate without depending on legacy `GatewayRequestContext`
   - Preserve compatibility mapping only as a projection

2. **Canonical settlement closure across all request paths**
   - Finish `/v1/moderations`
   - Finish synchronous image, audio, video, and music routes
   - Add stream-final settlement and release handling
   - Add provider-callback reconciliation entry points

3. **Commerce to canonical account migration**
   - Stop treating project quota as the commercial source of truth
   - Pay into canonical accounts and benefit lots
   - Project quota becomes an optional derived control artifact

4. **Real PostgreSQL canonical kernel**
   - Implement identity kernel
   - Implement account kernel CRUD
   - Implement hold and settlement transaction paths

5. **Admin and portal canonical finance read models**
   - account balance
   - active holds
   - benefit-lot posture
   - settlement explorer
   - route-level charge evidence
   - effective pricing view

6. **Real payment adapters**
   - Stripe server checkout plus webhook verification
   - Alipay server integration plus notify verification
   - WeChat Pay server integration plus notify verification
   - idempotent callback ingestion and payment evidence persistence

### P1: Required for strong commercial product quality

1. **Canonical pricing-plan and pricing-rate governance**
   - versioned tariff control in admin
   - effective customer pricing in portal
   - historical settlement replay against stored plan versions

2. **Canonical marketing write path**
   - template CRUD
   - batch issuance
   - redemption operations
   - invite and referral programs
   - campaign lifecycle and auditability

3. **Durable multimodal async kernel**
   - async jobs
   - retry and callback handling
   - generated asset records
   - admin and portal job centers

4. **Platform plugin inventory**
   - installed plugin classes
   - compatibility snapshots
   - health and revision state
   - rollout and feature-flag posture per module

5. **Finance operations**
   - refund and reversal workflow
   - reconciliation explorer
   - finance export
   - chargeback and exception posture

### P2: Required for advanced enterprise and global commercialization

1. **Enterprise identity**
   - OAuth
   - SSO
   - multi-workspace membership
   - delegated admin posture

2. **Database expansion**
   - real MySQL driver
   - real LibSQL driver

3. **Global commercial operations**
   - tax and invoice workflow
   - regional commerce posture
   - dunning and collections
   - enterprise contract and entitlement posture

## Recommended Execution Order

If only one path is chosen, the highest-value sequence is:

1. canonical gateway auth bridge
2. canonical settlement for remaining synchronous routes
3. commerce migration from project quota to canonical accounts
4. PostgreSQL canonical parity
5. admin and portal finance cutover
6. real payment provider adapters
7. async media job kernel
8. pricing-plan governance
9. canonical marketing write path
10. platform plugin inventory

## Final Assessment

The current application is:

- **architecturally strong**
- **feature-rich enough for continued pilot and internal hardening**
- **not yet complete enough for a polished commercial launch**

Its biggest advantage over many routers is that the architecture is already good enough to finish correctly.

Its biggest risk is that the remaining gaps are exactly the ones that create commercial incidents:

- wrong charges
- missing charges
- incompatible payment evidence
- weak operator reconciliation
- incomplete async media lifecycle

The system is now close enough that perfection no longer comes from large redesigns.

Perfection comes from finishing the last commercial kernels cleanly and refusing to widen the legacy compatibility paths further.
