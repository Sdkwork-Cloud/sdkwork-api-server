# Commercial System Gap Assessment And Target Solution

**Status:** architecture review completed against the local source tree on 2026-04-03

**Goal:** assess whether `sdkwork-api-router` already meets the bar of an advanced commercial API router platform, identify the remaining gaps precisely, and define the best target-state solution without carrying forward historical compatibility compromises.

## Executive Verdict

`sdkwork-api-router` is now a strong platform foundation, but it is **not yet** at the level of the strongest commercial AI gateway systems.

The current codebase is already ahead of many open-source routers in three important ways:

- the architecture is plugin-first and modular instead of monolithic
- multimodal API surface breadth is already strong
- admin and portal are already package-split React products with route and module metadata

But it is still behind the most advanced commercial systems in the places that matter most for revenue correctness, operator control, and production durability:

- request-path settlement correctness
- durable async media operations
- commercial control-plane completeness
- platform-wide plugin governance
- multi-database production parity

The right conclusion is not "rewrite it." The right conclusion is:

- keep the current architecture
- finish the missing commercial kernels
- tighten backend and product governance until the system becomes a true commercial platform

## Verified Strengths Already Present

The current source tree already proves several strong commercial foundations.

### 1. Plugin-first backend direction is real

Verified in source:

- `StorageDriverRegistry` exists in `crates/sdkwork-api-storage-core/src/lib.rs`
- `CacheDriverRegistry` exists in `crates/sdkwork-api-cache-core/src/lib.rs`
- builtin quota and billing policy registries exist in:
  - `crates/sdkwork-api-policy-quota/src/lib.rs`
  - `crates/sdkwork-api-policy-billing/src/lib.rs`
- runtime store and cache registry wiring exists in `crates/sdkwork-api-app-runtime/src/lib.rs`

This is stronger than the typical monolithic router shape and should remain the governing architecture.

### 2. Broad multimodal API coverage already exists

Verified in source:

- gateway route surface is centralized in `crates/sdkwork-api-interface-http/src/lib.rs`
- app-gateway multimodal operations already include:
  - image generation in `crates/sdkwork-api-app-gateway/src/lib.rs`
  - transcription in `crates/sdkwork-api-app-gateway/src/lib.rs`
  - music generation in `crates/sdkwork-api-app-gateway/src/lib.rs`
  - video generation in `crates/sdkwork-api-app-gateway/src/lib.rs`
- HTTP tests already exist for images, music, videos, transcriptions, responses, realtime, vector stores, uploads, and webhooks under `crates/sdkwork-api-interface-http/tests`

Compared with many API routers, the surface area is already commercially useful.

### 3. Canonical identity and account kernels now exist

Verified in source:

- canonical identity records and `GatewayAuthSubject` in `crates/sdkwork-api-domain-identity/src/lib.rs`
- canonical account records in `crates/sdkwork-api-domain-billing/src/lib.rs`
- `IdentityKernelStore` and `AccountKernelStore` in `crates/sdkwork-api-storage-core/src/lib.rs`
- SQLite canonical schema and CRUD in `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- account balance projection, hold planning, and payable-account resolution in `crates/sdkwork-api-app-billing/src/lib.rs`

This is the correct direction for a real commercial settlement kernel.

### 4. Admin and portal package modularity is already real

Verified in source:

- admin route manifest and module metadata in `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routeManifest.ts`
- portal route manifest and module metadata in `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/routeManifest.ts`
- admin and portal type-level manifest contracts in:
  - `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
  - `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`

This confirms the React package split is already aligned with the desired product-module direction.

## The Remaining Gaps To Advanced Commercial Quality

The remaining gaps are not cosmetic. They are the exact pieces that separate a good architecture from a finished commercial platform.

## Gap 1: Request-path billing and settlement is not transaction-safe yet

**Severity:** critical

Current verified state:

- canonical subject resolution now exists
- canonical payable-account resolution now exists
- account balance projection and hold planning now exist
- the live gateway still enforces legacy project quota and coarse ledger paths in `crates/sdkwork-api-interface-http/src/lib.rs`

The structural problem:

- there is still no public commercial transaction seam in `sdkwork-api-storage-core`
- the request path still does not execute:
  - hold creation
  - hold allocation persistence
  - ledger emission
  - release or capture
  - immutable settlement creation

Why this blocks commercial maturity:

- without a stable transaction boundary, the gateway cannot guarantee consistent balance mutation
- partial failures would create charge leakage or inventory corruption
- advanced commercial systems treat this as non-negotiable

### Target solution

Add a canonical account-command transaction layer:

- storage-core transaction abstraction for account-kernel command execution
- app-billing command service for:
  - create hold
  - release hold
  - capture hold
  - emit ledger entries and allocations
  - create request settlement
- gateway admission cutover only after this layer is fully verified

Current progress:

- SQLite-backed account-kernel command batching now exists
- `sdkwork-api-app-billing` now ships create-hold, capture-hold, and release-hold orchestration with request-meter-fact and settlement persistence
- gateway admission and settlement cutover are still pending, so this gap is reduced but not closed

## Gap 2: Multimodal async job kernel is still missing

**Severity:** critical

Current verified state:

- sync and relay API coverage for image, audio, video, music, and webhooks already exists
- the repository docs repeatedly call out the missing durable async media job subsystem
- current code search still does not show a shipped canonical backend job kernel with dedicated records, services, and control-plane APIs

Why this matters:

- advanced commercial systems do not stop at endpoint parity
- they provide durable job state, reconciliation, retries, artifacts, callbacks, and operator visibility
- `new-api` still has practical product-density advantages here

### Target solution

Add a multimodal async execution kernel with:

- `ai_async_job`
- `ai_async_job_attempt`
- `ai_generated_asset`
- `ai_provider_callback_event`
- `ai_async_job_webhook_delivery`
- app-job orchestration service
- operator and tenant job workbenches

This should become the canonical path for long-running image, video, audio, and music workloads.

## Gap 3: Admin and portal are still only partially commercialized

**Severity:** high

Current verified state:

- admin and portal already expose route manifests, API key groups, routing profiles, billing events, and package-first modules
- but the audit baseline still shows that they do not yet operate on the canonical account, pricing, hold, and settlement kernel end to end

Why this matters:

- advanced systems win through control-plane density, not only data-plane breadth
- operators need pricing governance, settlement explorer, reconciliation, plugin compatibility, rollout control, and media job operations
- tenants need account history, settlement evidence, recharge posture, pricing clarity, and asset/job visibility

### Target solution

Admin should gain:

- pricing plan and pricing rate governance
- account governance
- benefit-lot governance
- hold and settlement explorer
- plugin inventory and compatibility center
- async job operations center
- feature-flag and rollout governance per module

Portal should gain:

- account-balance views backed by canonical account kernel
- settlement and recharge evidence
- job center for media generation lifecycle
- transparent pricing and chargeback posture
- self-service routing, cost, and SLA posture views

## Gap 4: Plugin platform governance is not finished end to end

**Severity:** high

Current verified state:

- registries exist for storage, cache, billing policy, and quota policy
- extension runtime installation and rollout are real
- admin and portal already expose module metadata

What is still missing:

- backend product-module registry
- generic plugin inventory across all plugin classes
- compatibility snapshots
- module-level feature flags
- backend route ownership by module
- plugin health and config revision center

### Target solution

Turn the current plugin-first direction into a full platform contract:

- backend module registration records
- module rollout records
- plugin compatibility evidence
- unified inventory across:
  - storage drivers
  - cache drivers
  - policy plugins
  - runtime plugins
  - product modules

## Gap 5: Multi-database story is still not production complete

**Severity:** high

Current verified state:

- SQLite is the strongest canonical backend today
- PostgreSQL has meaningful schema and broad runtime code, but the new commercial kernel is not yet fully parity-implemented there
- MySQL and LibSQL crates currently only prove dialect identity, not real runtime production behavior

Why this matters:

- advanced commercial systems do not leave their core commercial kernel tied to one concrete backend
- once settlement is real, dialect parity becomes a platform requirement

### Target solution

Finish storage parity in this order:

1. PostgreSQL canonical account and identity CRUD parity
2. PostgreSQL transaction-backed billing command execution
3. MySQL real driver
4. LibSQL real driver

Until then, unsupported dialects must continue failing honestly.

## Gap 6: Pricing reproducibility and finance workflows are not complete

**Severity:** high

Current verified state:

- pricing plan and pricing rate records exist in the canonical billing model
- billing events already separate upstream cost and customer charge
- API key groups can carry default accounting mode

What is still missing:

- runtime pricing-plan resolution on live request settlement
- settlement snapshots that can reproduce historical charges exactly
- reconciliation and finance-export projections
- correction flows for late usage, async usage, and webhook-delayed adjustments

### Target solution

Add:

- deterministic pricing-plan resolver in app-billing
- settlement-time price snapshot attachment
- reconciliation settlement variants
- export projections for finance and BI

## Gap 7: Enterprise governance is not yet fully evidenced

**Severity:** medium

Current verified state:

- admin and portal module manifests already describe permissions
- desktop shells use Tauri capability and permission models

What is still not yet evidenced as a full commercial governance layer:

- centralized backend feature-flag governance
- tenant-visible compliance policy posture
- operator-grade audit export around settlement and plugin operations
- generalized abuse, residency, and compliance policy plugins

### Target solution

Promote governance to a first-class platform layer:

- module feature-flag registry
- policy attachment to tenant, project, group, and request
- exportable audit evidence for commercial operations
- compliance and abuse policy plugin seams

## Maturity Scorecard

This is the best concise maturity summary for the current system.

- architecture quality: strong
- plugin direction: strong
- multimodal surface coverage: strong
- canonical commercial data model: good
- live settlement correctness: incomplete
- multimodal async operations: incomplete
- admin and portal commercial density: partial
- multi-database production parity: partial
- enterprise control-plane governance: partial

## Target-State Solution

The best target state is a five-layer commercial platform:

### Layer 1: Canonical identity and payable subject

- `GatewayAuthSubject`
- canonical API keys and bindings
- active primary payable account per user

### Layer 2: Transaction-safe commercial account kernel

- balance projection
- hold planning
- hold mutation
- settlement capture and release
- immutable ledger and allocation evidence

### Layer 3: Multimodal execution kernel

- synchronous request path for fast operations
- durable async job path for long-running media and webhook-driven workloads
- unified asset lifecycle

### Layer 4: Commercial control plane

- pricing governance
- settlement explorer
- reconciliation
- plugin inventory
- rollout and feature flags
- account and media operations

### Layer 5: Platform portability and governance

- backend driver parity
- plugin compatibility management
- policy plugins for risk, residency, and compliance
- export and analytics projections

## Best Execution Order

The best execution order is:

1. transaction-safe account mutation kernel
2. gateway cutover to canonical admission and settlement
3. admin and portal cutover to canonical account and settlement views
4. async multimodal job kernel
5. plugin inventory and backend module registry
6. PostgreSQL parity, then MySQL and LibSQL
7. reconciliation, finance export, and enterprise governance polish

This order is optimal because it fixes correctness before density, and density before peripheral portability.

## Final Assessment

The current system is already architecturally better than many routers in the market.

It is **not** yet a finished best-in-class commercial system.

The missing work is now sharply defined:

- finish the commercial settlement kernel
- add the async multimodal operations kernel
- finish the commercial control plane
- complete platform governance and dialect parity

That is the shortest path from the current strong foundation to an elite commercial API router platform.
