# Router Implementation Audit And Upgrade Plan

**Status:** checked against local source on 2026-04-03

**Scope:** backend runtime, storage, billing, multimodal gateway surface, admin product, portal product, plugin seams, and the local `new-api` snapshot under `.external/new-api`

**Constraint note:** a local `new-api` source snapshot exists in this workspace, but there is no checked-in OpenRouter upstream source tree under `.external`. Current OpenRouter comparisons therefore remain grounded in the approved internal design and the repository's existing OpenRouter-compatible provider, channel, and routing semantics, not a line-by-line upstream source diff.

## Executive Verdict

`sdkwork-api-router` is now structurally stronger than `new-api` in extensibility, testability, and plugin-oriented architecture, but it is still behind the target commercial end state in four important areas:

- the commercial account and settlement kernel is now partially shipped in SQLite, but not yet wired end to end through gateway, admin, and portal
- multimodal async job orchestration is still missing as a durable backend subsystem
- multi-database support is still only fully real for the legacy surfaces; the new account kernel is real in SQLite and schema-only in PostgreSQL
- plugin and product modularity is now real at the React route level, but not yet complete across backend registry, control plane inventory, and rollout governance

That means the current system is already a strong programmable gateway foundation, but not yet a finished commercial cloud gateway.

## What Is Already Better Than `new-api`

Compared with the local `new-api` snapshot, the current repository is ahead in architecture quality:

- domain, app, storage, interface, runtime, and extension boundaries are explicit instead of being folded into one fast-moving product code path
- storage already has facet-style contracts and a driver registry seam
- cache already has a dedicated registry seam
- routing, quota, and billing policies already have dedicated plugin-style registries
- rollout, reload, and runtime supervision are first-class backend concerns
- admin and portal now stay package-based React products while exposing module metadata needed for plugin-first composition

This is the right long-term architecture. It should not be rewritten into a monolith.

## What `new-api` Still Does Better

The local `new-api` snapshot still demonstrates stronger product-density around multimodal task coverage. Its task adaptor tree under `.external/new-api/relay/channel/task/` already includes provider-specific task entry points such as `suno`, `kling`, and `vertex`.

The key lesson is not to copy its transport shape directly. The useful lesson is:

- broaden media provider coverage quickly
- model image, video, and music as first-class product capabilities
- provide operator-friendly workflow density for day-to-day media operations

Our repository already has broad OpenAI-compatible media routes, but it still lacks the durable async job kernel that turns those routes into a complete commercial media platform.

## What The OpenRouter Direction Still Demands

The approved routing and billing design already captures the right OpenRouter-inspired direction:

- provider and model capability metadata must be first-class
- routing policy must be observable and configurable
- cost, latency, fallback, privacy, and cache effects must be visible inputs and outputs
- multimodal discovery and generation evidence must be part of the product, not an afterthought

The current codebase has made real progress here through routing profiles, compiled routing snapshots, provider health, fallback evidence, and billing event summaries. What is still missing is the full commercial kernel underneath those views.

## Verified Strengths In The Current Codebase

The following foundations are real and already landed:

- storage facets and registry-driven selection exist
- cache driver registry exists and is wired into runtime initialization
- routing, quota, and billing policy registries exist
- multimodal billing dimensions exist for image, audio, video, and music usage
- admin and portal already expose API key groups, routing profiles, compiled routing snapshots, and billing event summaries
- admin and portal now expose package-first product module manifests carrying capability, permission, navigation, and loading metadata
- standalone runtime now fails honestly for unsupported storage dialects and explicitly lists supported dialects

## Gaps That Still Block A Finished Commercial System

### 1. Commercial account kernel is not implemented end to end

The canonical aggregated-cloud design defines a much stronger billing and identity system:

- `tenant_id`, `organization_id`, and `user_id` as `BIGINT`
- a real `ai_account` kernel
- benefit lots, holds, ledger entries, request facts, pricing plans, pricing rates, and settlements

The actual production code is still materially behind that target:

- domain contracts now exist for the canonical kernel
- SQLite migrations and SQLite CRUD now exist for `ai_account`, `ai_account_benefit_lot`, `ai_account_hold`, `ai_account_ledger_entry`, `ai_request_meter_fact`, `ai_request_meter_metric`, `ai_request_settlement`, `ai_pricing_plan`, and `ai_pricing_rate`
- current commerce, admin, portal, and gateway flows remain heavily workspace, project, and string-id oriented
- PostgreSQL mirrors the schema, but does not yet expose real account-kernel CRUD behavior
- there is still no shipped end-to-end implementation that resolves auth subject, creates holds, persists request facts, prices a request, and settles it through the new kernel

This is the single highest-value gap left in the system.

### 2. Billing is improved, but still compatibility-grade above the storage layer

The billing domain is better than before, but it is not yet a true cloud settlement kernel.

Current strengths:

- multimodal dimensions already exist in billing events
- upstream cost and customer charge are already separate
- API key groups and routing profiles can already participate in billing evidence

Current weakness:

- the gateway still records coarse usage, billing events, and ledger entries on compatibility-era paths
- `sdkwork-api-app-billing` now has the first clean-slate `AccountKernelStore` read-model and hold-planning APIs, and the SQLite path now also has transaction-backed command orchestration for create-hold, capture-hold, and release-hold flows with request-meter-fact and settlement persistence
- there is still no immutable meter-fact to settlement pipeline in the request path
- the app layer now has an account hold and release lifecycle, but the HTTP gateway still has not cut over to it
- no pricing plan resolution flow that reproduces historical charges exactly at settlement time
- no benefit-lot application model for prepaid, promo, or package offsets in business logic

### 3. Multimodal async job execution is still missing

The approved design explicitly calls for an async media job model, but current code search still shows that this is mainly present in design docs rather than concrete backend tables, services, and APIs.

Practical consequence:

- images, video, and music can be routed at the API surface
- but long-running generation, webhook reconciliation, asset lifecycle, and operator job visibility are not yet a complete system

This is where `new-api` currently has the practical product-density advantage.

### 4. MySQL and LibSQL are not real runtime backends yet

Current status is still partial:

- config recognizes `mysql` and `libsql`
- placeholder crates exist for `sdkwork-api-storage-mysql` and `sdkwork-api-storage-libsql`
- standalone runtime actually wires only SQLite and PostgreSQL

This pass tightened the runtime honesty boundary so unsupported dialects fail with explicit supported-dialect messaging, which is the correct short-term behavior.

What is still missing is the real driver implementation itself.

### 5. Product module standard is now real in the frontend, but not yet end to end

This pass formalized admin and portal module manifests with:

- plugin identity
- capability tags
- required permissions
- navigation metadata
- loading and prefetch policy

That is a real upgrade and it is the correct package-first path.

The next missing step is backend parity:

- a backend product-module registry
- module-level feature flags and staged rollout
- backend route ownership by module
- module migration ownership
- admin evidence for installed product modules and their health or compatibility state

### 6. Plugin inventory is still runtime-centric, not platform-wide

There is already real extension installation support for provider runtimes and extension instances. That is good.

The gap is that plugin inventory is not yet generalized across all plugin classes:

- storage drivers
- cache drivers
- policy plugins
- product modules

The control plane should eventually expose installed plugins, compatibility state, health, and configuration revision across all of those seams, not only provider-runtime installations.

### 7. Admin and portal are not yet finished commercial control planes

Admin and portal are already much better than before, but the next commercial upgrade layer still needs:

- pricing plan and pricing rate governance
- settlement explorer and reconciliation tooling
- user account, hold, benefit-lot, and balance governance
- async job center for image, video, audio, and music tasks
- plugin inventory and compatibility center
- staged rollout and feature-flag management per product module

## What Was Landed In This Audit Pass

This pass closes four important structural gaps:

### Commercial account kernel contract freeze

The repository now has a first real implementation step for the aggregated-cloud account design:

- canonical bigint-oriented gateway identity aliases and `GatewayAuthSubject` now exist in the identity domain
- billing domain now includes canonical account, benefit-lot, hold, ledger, settlement, pricing-plan, and pricing-rate records
- usage domain now includes canonical request-meter fact and metric records
- `storage-core` now exposes an `AccountKernelStore` facet with explicit unsupported responses for unimplemented dialect behavior, which gives the commercial kernel a formal storage seam without pretending every backend is already finished

This is not the final business kernel, but it is the right intermediate state:

- domain contracts are now explicit instead of being implicit in design docs only
- storage extension points now exist for the new kernel
- concrete database schemas and gateway execution still remain to be implemented

### Canonical account schema landing

The storage layer is now beyond design-only status for the canonical account kernel:

- SQLite migrations now create the first canonical `ai_account`, `ai_account_benefit_lot`, `ai_account_hold`, `ai_account_ledger_entry`, `ai_request_meter_fact`, `ai_request_meter_metric`, `ai_request_settlement`, `ai_pricing_plan`, and `ai_pricing_rate` tables
- the same canonical table set is now mirrored in PostgreSQL migrations
- SQLite migration tests now verify table creation, bigint-style scope columns, `organization_id DEFAULT 0`, and key operational indexes
- the account-kernel storage facet was adjusted to avoid a blanket implementation that would have blocked dialect-specific implementations later
- SQLite `AccountKernelStore` now exposes real CRUD and round-trip decoding for the canonical account tables
- SQLite regression coverage now verifies both the schema and a full account-kernel round trip through the storage seam

This materially improves the database architecture, but the business path is still incomplete because:

- only SQLite currently exposes the new account-kernel CRUD path
- PostgreSQL still mirrors only the schema unless a later pass lands real storage methods
- the gateway still writes compatibility-era `ai_usage_records`, `ai_billing_events`, and `ai_billing_ledger_entries`
- admin and portal still read the old summary surfaces rather than the new account kernel directly
- commerce still centers on `project_id`, `granted_units`, `bonus_units`, and `included_units` rather than payable user accounts and benefit lots

### Canonical identity kernel bridge

The identity side is now also beyond design-only status for the canonical gateway subject model:

- the identity domain now includes explicit canonical records for `ai_user`, `ai_api_key`, and `ai_identity_binding`
- `storage-core` now exposes an `IdentityKernelStore` facet instead of forcing canonical identity behavior into the legacy admin surface
- SQLite migrations now create `ai_user`, `ai_api_key`, and `ai_identity_binding` with bigint scope columns and operational indexes
- PostgreSQL now mirrors the same canonical identity schema and index set
- SQLite now exposes real CRUD and round-trip decoding for canonical user, API key, and identity binding records
- `sdkwork-api-app-identity` can now resolve a `GatewayAuthSubject` from a canonical API key record while preserving the older workspace-string `GatewayRequestContext` in parallel

This is the right bridge state, but the full request path is still incomplete because:

- the HTTP gateway still authenticates against the legacy workspace-string request context instead of the canonical subject resolver
- the HTTP gateway still does not consume the canonical subject-to-account resolver on live admission paths
- hold, release, and settlement mutations still do not execute inside a transactional account-kernel orchestration layer

### Canonical payable-account subject resolution

The account side now has the first real bridge from canonical auth subject to canonical payable account:

- `AccountKernelStore` now exposes owner-scope lookup for `(tenant_id, organization_id, user_id, account_type)`
- SQLite implements the indexed owner-scope query against `ai_account`
- `sdkwork-api-app-billing` now resolves the active primary `ai_account` for a `GatewayAuthSubject`
- resolution returns `None` when a payable account has not yet been provisioned
- suspended or closed primary accounts now fail closed instead of being silently treated as chargeable

This is the correct precondition for the next billing phase, but it is still only a bridge layer because:

- the HTTP gateway still admits requests on the legacy workspace-string path
- request execution still does not create transactional holds or settlements in the canonical kernel
- PostgreSQL still lacks real account-kernel CRUD parity beyond schema mirroring

### Product-module manifest uplift

Admin and portal now expose richer package-first manifests instead of only lightweight route metadata.

The new manifest layer now includes:

- plugin kind
- capability tags
- required permissions
- navigation descriptors
- lazy-load policy and prefetch policy

This preserves the required React package topology while making module boundaries explicit and machine-readable.

### Runtime storage support guardrails

Standalone runtime storage selection now reports supported dialects explicitly when the configured database dialect is not implemented.

This prevents the system from pretending that MySQL or LibSQL are ready when only SQLite and PostgreSQL are actually wired.

## Recommended Execution Order

The highest-value next order is:

### Phase 1: Commercial account kernel

Implement the canonical account and billing core first:

- `ai_account`
- `ai_account_benefit_lot`
- `ai_account_hold`
- `ai_account_ledger_entry`
- `ai_pricing_plan`
- `ai_pricing_rate`
- `ai_request_meter_fact`
- `ai_request_settlement`

This phase should start with domain and storage contracts plus compatibility projections, not UI work.

Immediate next step after this audit update:

- extend the new `sdkwork-api-app-billing` account service from subject resolution, read-models, and hold planning into real hold creation, release, and settlement mutations
- keep PostgreSQL storage parity as the next storage follow-up so the new kernel remains database-portable

### Phase 2: Gateway settlement flow

Wire request execution into:

- canonical auth subject resolution
- meter fact creation
- pricing resolution
- hold and settlement transitions
- immutable audit evidence

### Phase 3: Async multimodal job kernel

Add:

- async generation job records
- asset lifecycle records
- polling and webhook reconciliation
- operator-facing and tenant-facing job workbenches

This is the place to absorb the strongest practical lesson from the local `new-api` snapshot.

### Phase 4: Backend product-module registry and plugin inventory

Make the plugin-first standard end to end:

- backend product module registration
- feature flags and staged rollout
- generic plugin inventory
- compatibility snapshots
- plugin health and config revision evidence

### Phase 5: Real MySQL and LibSQL support

Only after the commercial kernel is stable should more databases be added as full production drivers.

Until then, explicit unsupported status is better than false confidence.

## Final Assessment

The current architecture direction is correct.

The repository should keep:

- plugin-first architecture
- package-first admin and portal composition
- registry-driven infrastructure seams
- routing and billing evidence model

The repository should now concentrate on the missing business kernel rather than widening the UI shell alone.

The main conclusion is simple:

- architecture foundation: strong
- plugin direction: correct
- multimodal surface: broad
- business kernel: not complete yet
- database portability story: honest but not finished

That is the right point to be at before the next phase. The system does not need a rewrite. It needs the commercial account, settlement, and async job layers completed on top of the current architecture.
