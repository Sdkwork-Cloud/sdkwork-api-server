# Aggregated Cloud Gateway Account System Design

## Goal

Design a production-grade account, identity, metering, pricing, settlement, and reporting model for `sdkwork-api-router` so the router can evolve into a standard aggregated cloud gateway for global AI providers and future multimodal APIs.

The design must satisfy these hard requirements:

- every physical table name starts with `ai_`
- every table except `ai_tenant` includes `tenant_id` and `organization_id`
- `tenant_id`, `organization_id`, and `user_id` are `BIGINT`
- `organization_id` is `BIGINT NOT NULL DEFAULT 0`
- the real billing subject is the user account, not the tenant
- the tenant only provides management, governance, reporting, and aggregation scopes
- the gateway must support both `API Key` and `JWT Token` access
- JWT handling must be compatible with the Java `PlusAuthToken` structure already used in the wider workspace
- one user can own multiple API keys
- every usage and billing record must be attributable to `tenant_id + organization_id + user_id + auth_type + api_key_id(nullable)`
- the model must support future expansion across text, image, audio, video, music, realtime, storage, search, and tool-execution billing

## Why The Current Router Billing Model Is Not Enough

The current repository already has useful building blocks:

- provider catalog, routing, usage, quota, and ledger primitives
- OpenAI-compatible, Anthropic-compatible, and Gemini-compatible surfaces
- portal and admin products with billing-oriented views
- `ai_model_price` for coarse provider pricing

The current model is still structurally demo-grade:

- `usage` is still centered on coarse `units`, `amount`, and a few token columns
- `billing` is effectively quota plus summary ledger, not an auditable settlement kernel
- account ownership is not modeled around `tenant + organization + user`
- `API Key` and `JWT` are not yet unified into one gateway billing subject
- price versioning is not strong enough to reproduce historical charges exactly
- future multimodal dimensions would force repeated schema churn

This is acceptable for a prototype but not for a standard aggregated cloud gateway.

## Industry Reference Patterns

The professional cloud platforms converge on the same structural ideas:

- AWS separates payer scope, linked-account scope, detailed cost-and-usage lines, tags, credits, and custom billing views.
- Azure separates billing account, billing profile, invoice section, subscription, cost scope, and credit application.
- Google Cloud separates billing accounts, projects, detailed usage export, pricing export, and commitment metadata export.
- Alibaba Cloud and Tencent Cloud separate payable amount, discounts, resource-package offsets, tag-based cost allocation, and payer/use-account analysis.

The common design rules are:

1. identity is not the same thing as billing subject
2. raw usage facts are not the same thing as billed results
3. current prices are not allowed to rewrite historical settlements
4. discounts, grants, and packages are modeled as separate accounting objects
5. reporting views are projections over immutable detailed lines

This design follows that professional pattern while adapting it to an AI gateway where the payable subject is the user account.

## Design Options

### Option A: Simple Wallet

- one user balance field
- one usage row per request
- one ledger row per request

Pros:

- fastest to ship
- lowest schema count

Cons:

- poor auditability
- weak support for grants and scoped packages
- fragile under multimodal growth

### Option B: Layered Cloud-Gateway Settlement Model

- user-scoped accounts
- immutable benefit lots
- hold then settle lifecycle
- row-based metrics
- versioned pricing plans
- compatibility projections for legacy tables

Pros:

- closest to professional cloud billing patterns
- supports `JWT` and `API Key` through one account subject
- supports current and future modalities without redesigning the account kernel
- supports audits, refunds, reconciliations, and profitability views

Cons:

- more initial tables and workflow complexity

### Option C: Full Financial ERP From Day One

- double-entry ledger
- receivables, invoices, taxes, finance close, GL integration

Pros:

- finance-grade completeness

Cons:

- too heavy for the current product stage
- slows down the gateway product core

## Recommendation

Adopt **Option B** now and leave explicit extension seams toward Option C.

This gives the router a professional cloud-gateway account kernel without overfitting to a full finance ERP before the gateway itself is complete.

## Scope And Phased Program

The complete aggregated cloud gateway should be built in these phases:

1. account and identity kernel
2. metering and pricing kernel
3. hold and settlement execution in the gateway path
4. admin and portal governance surfaces
5. analytics, reconciliation, profitability, and finance-export projections

This document defines the final target architecture and the minimum phase ordering to reach it safely.

## Core Design Decisions

### 1. User Account Is The Only Payable Subject

- `tenant` is a governance and reporting scope
- `organization` is a secondary grouping scope under a tenant
- `user` is the real payable subject
- every request resolves to exactly one user account

Tenant-level analysis remains mandatory, but tenant-level direct charging is not part of the current system design.

### 2. JWT And API Key Are Two Entry Modes Into One Account System

- `JWT Token` requests and `API Key` requests must converge into the same request context model
- both modes must resolve the same `tenant_id + organization_id + user_id`
- both modes must support per-credential usage and billing analysis
- `api_key_id` is nullable for JWT requests but required for API-key requests

### 3. Every Business Table Except `ai_tenant` Carries `tenant_id` And `organization_id`

Standard columns:

- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`

This is a hard schema rule for isolation, filtering, indexing, and reporting.

### 4. Bigint IDs Are The Standard

Standard identity columns:

- `tenant_id BIGINT`
- `organization_id BIGINT`
- `user_id BIGINT`

Recommendation:

- all new major surrogate keys should also be `BIGINT`
- generated by a Snowflake-style or segment-style ID service instead of database auto-increment

This keeps the system storage-agnostic and compatible with distributed ingestion and cross-store replication.

### 5. Balance Is A Projection, Not The Source Of Truth

The source of truth is the combination of:

- `ai_account_benefit_lot`
- `ai_account_hold`
- `ai_account_ledger_entry`
- `ai_request_settlement`

Displayed balances are projections over those immutable or append-only records.

### 6. Metering Is Row-Based, Not Column-Churn-Based

The gateway must normalize provider usage into metric rows instead of adding a new column for every new provider field.

### 7. Pricing Must Be Versioned And Snapshotted

Every billed request must retain the exact pricing-plan version used at settlement time.

## Identity And Authentication Model

### Java JWT Compatibility

The wider Java workspace uses `PlusAuthToken` semantics with at least these claims:

- `tenantId`
- `organizationId`
- `userId`
- `platform`
- `owner`
- `createdAt`
- `expires`
- `type`
- `sub`

The router must support:

1. unverified claim extraction for scope discovery
2. tenant-and-organization aware secret resolution
3. verified JWT parsing into a canonical gateway auth context

### Canonical Gateway Auth Context

Every accepted gateway request must resolve a canonical auth context:

- `tenant_id BIGINT`
- `organization_id BIGINT`
- `user_id BIGINT`
- `auth_type VARCHAR`
- `api_key_id BIGINT NULL`
- `api_key_hash VARCHAR NULL`
- `jwt_subject VARCHAR NULL`
- `platform VARCHAR NULL`
- `owner VARCHAR NULL`
- `request_principal VARCHAR`

`request_principal` is the stable normalized identity string used for logging and audit evidence.

## Canonical Account Semantics

### Account Concepts

- one user has one primary `ai_account`
- one account can hold multiple benefit lots
- every request can create one hold and one settlement envelope
- every financial mutation is emitted as immutable ledger entries

### Benefit Lot Types

The initial catalog should support:

- `cash_credit`
- `promo_credit`
- `request_allowance`
- `token_allowance`
- `image_allowance`
- `audio_allowance`
- `video_allowance`
- `music_allowance`

### Balance Definitions

- `available_balance`: free assets available for new holds
- `held_balance`: reserved assets under active holds
- `consumed_balance`: settled charges already captured
- `expired_balance`: lots or allocations that expired
- `refunded_balance`: refunded or compensated assets
- `grant_balance`: issued assets before net consumption

### Spend Priority

Default spend policy:

1. earliest expiry first
2. narrowest scope first
3. promotional and package assets before cash
4. lower acquisition cost first when still tied

This policy is the best default for a gateway product because it preserves customer value without losing operator auditability.

## Canonical Meter Catalog

The initial metric catalog should include:

- `request.count`
- `token.input`
- `token.output`
- `token.cache_read`
- `token.cache_write`
- `token.reasoning`
- `token.tool_input`
- `token.thought`
- `image.count`
- `image.pixel`
- `audio.input_token`
- `audio.output_token`
- `audio.second`
- `video.second`
- `music.second`
- `realtime.minute`
- `search.request`
- `storage.gb_day`

This metric set covers current industry patterns while leaving room for future API families.

## Provider Pricing Compatibility

The router must support pricing structures commonly exposed by major model providers:

- OpenAI-style input, cached input, output, reasoning, image, audio, and video pricing
- Anthropic-style input, output, cache creation, and cache read pricing
- Gemini-style prompt, candidate, thought, cached, and tool-use token pricing
- DeepSeek, Qwen, GLM, Moonshot, MiniMax, Tencent Hunyuan, and similar vendor models with token, request, or modality-based pricing

The exact price amounts change over time, so the data model must store generalized rate rules instead of provider-specific hardcoded fields.

## Physical Table Model

### Table Conventions

Except for `ai_tenant`, every table includes at least:

- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `created_at_ms BIGINT NOT NULL`
- `updated_at_ms BIGINT NOT NULL`

For immutable event tables:

- keep `created_at_ms`
- omit mutable updates when possible
- never delete financial rows

For business-master tables:

- prefer `status` flags to hard deletes

### 1. Identity And Governance

#### `ai_tenant`

Purpose:

- top-level governance scope

Key columns:

- `tenant_id BIGINT PRIMARY KEY`
- `tenant_name VARCHAR`
- `status VARCHAR`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

#### `ai_user`

Purpose:

- user identity and billing ownership

Key columns:

- `user_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `external_user_ref VARCHAR NULL`
- `username VARCHAR NULL`
- `display_name VARCHAR NULL`
- `email VARCHAR NULL`
- `status VARCHAR`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

Indexes:

- `(tenant_id, organization_id, user_id)`
- `(tenant_id, organization_id, email)`

#### `ai_api_key`

Purpose:

- machine credentials under a user

Key columns:

- `api_key_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `user_id BIGINT NOT NULL`
- `key_prefix VARCHAR`
- `key_hash VARCHAR`
- `display_name VARCHAR`
- `status VARCHAR`
- `expires_at_ms BIGINT NULL`
- `last_used_at_ms BIGINT NULL`
- `rotated_from_api_key_id BIGINT NULL`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

Unique constraints:

- unique `key_hash`

Indexes:

- `(tenant_id, organization_id, user_id, status)`

#### `ai_identity_binding`

Purpose:

- external identity bindings for JWT subject, issuer, external user references, or future identity providers

Key columns:

- `identity_binding_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `user_id BIGINT NOT NULL`
- `binding_type VARCHAR`
- `issuer VARCHAR NULL`
- `subject VARCHAR NULL`
- `platform VARCHAR NULL`
- `owner VARCHAR NULL`
- `external_ref VARCHAR NULL`
- `status VARCHAR`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

### 2. Accounts And Asset Lots

#### `ai_account`

Purpose:

- payable user account

Key columns:

- `account_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `user_id BIGINT NOT NULL`
- `account_type VARCHAR`
- `currency_code VARCHAR`
- `credit_unit_code VARCHAR`
- `status VARCHAR`
- `allow_overdraft BOOLEAN`
- `overdraft_limit DECIMAL(24,8)`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

Unique constraints:

- unique `(tenant_id, organization_id, user_id, account_type)`

#### `ai_account_policy`

Purpose:

- account-level controls and defaults

Key columns:

- `account_policy_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `account_id BIGINT NOT NULL`
- `hold_timeout_ms BIGINT`
- `estimate_multiplier DECIMAL(12,6)`
- `allow_incremental_capture BOOLEAN`
- `allow_negative_settlement BOOLEAN`
- `status VARCHAR`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

#### `ai_account_benefit_lot`

Purpose:

- immutable benefit issuance with mutable remaining state

Key columns:

- `lot_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `account_id BIGINT NOT NULL`
- `user_id BIGINT NOT NULL`
- `benefit_type VARCHAR`
- `source_type VARCHAR`
- `source_id BIGINT NULL`
- `scope_json JSON/TEXT`
- `original_quantity DECIMAL(24,8)`
- `remaining_quantity DECIMAL(24,8)`
- `held_quantity DECIMAL(24,8)`
- `priority INT`
- `acquired_unit_cost DECIMAL(24,8) NULL`
- `issued_at_ms BIGINT`
- `expires_at_ms BIGINT NULL`
- `status VARCHAR`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

Indexes:

- `(tenant_id, organization_id, account_id, status, expires_at_ms)`
- `(tenant_id, organization_id, user_id, benefit_type, status)`

#### `ai_account_balance_snapshot`

Purpose:

- query-optimized balance projection

Key columns:

- `balance_snapshot_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `account_id BIGINT NOT NULL`
- `user_id BIGINT NOT NULL`
- `available_balance DECIMAL(24,8)`
- `held_balance DECIMAL(24,8)`
- `consumed_balance DECIMAL(24,8)`
- `expired_balance DECIMAL(24,8)`
- `refunded_balance DECIMAL(24,8)`
- `grant_balance DECIMAL(24,8)`
- `snapshot_at_ms BIGINT`
- `updated_at_ms BIGINT`

Rule:

- this table is a projection only, never the financial source of truth

### 3. Holds, Ledger, And Allocation

#### `ai_account_hold`

Purpose:

- request pre-authorization header

Key columns:

- `hold_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `account_id BIGINT NOT NULL`
- `user_id BIGINT NOT NULL`
- `request_id BIGINT NOT NULL`
- `hold_status VARCHAR`
- `estimated_quantity DECIMAL(24,8)`
- `captured_quantity DECIMAL(24,8)`
- `released_quantity DECIMAL(24,8)`
- `expires_at_ms BIGINT`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

Unique constraints:

- unique `(tenant_id, organization_id, request_id)`

#### `ai_account_hold_allocation`

Purpose:

- lot-level reservation details under a hold

Key columns:

- `hold_allocation_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `hold_id BIGINT NOT NULL`
- `lot_id BIGINT NOT NULL`
- `allocated_quantity DECIMAL(24,8)`
- `captured_quantity DECIMAL(24,8)`
- `released_quantity DECIMAL(24,8)`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

#### `ai_account_ledger_entry`

Purpose:

- immutable account event ledger

Key columns:

- `ledger_entry_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `account_id BIGINT NOT NULL`
- `user_id BIGINT NOT NULL`
- `request_id BIGINT NULL`
- `hold_id BIGINT NULL`
- `entry_type VARCHAR`
- `benefit_type VARCHAR`
- `quantity_delta DECIMAL(24,8)`
- `balance_after DECIMAL(24,8) NULL`
- `source_type VARCHAR`
- `source_id BIGINT NULL`
- `notes VARCHAR NULL`
- `created_at_ms BIGINT`

Entry types:

- `issue`
- `hold_create`
- `hold_release`
- `settle_capture`
- `settle_refund`
- `expire`
- `adjustment`
- `reconcile`

#### `ai_account_ledger_allocation`

Purpose:

- mapping between ledger entries and lots

Key columns:

- `ledger_allocation_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `ledger_entry_id BIGINT NOT NULL`
- `lot_id BIGINT NOT NULL`
- `quantity_delta DECIMAL(24,8)`
- `created_at_ms BIGINT`

### 4. Metering And Pricing

#### `ai_meter_definition`

Purpose:

- canonical metric registry

Key columns:

- `metric_code VARCHAR PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `metric_name VARCHAR`
- `unit_kind VARCHAR`
- `aggregation_kind VARCHAR`
- `default_rounding_mode VARCHAR`
- `status VARCHAR`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

Note:

- for global built-in metrics, use `tenant_id = 0` and `organization_id = 0`

#### `ai_pricing_plan`

Purpose:

- pricing plan header with versioning

Key columns:

- `pricing_plan_version_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `plan_code VARCHAR`
- `plan_version INT`
- `plan_type VARCHAR`
- `scope_kind VARCHAR`
- `scope_ref_id BIGINT NULL`
- `display_name VARCHAR`
- `currency_code VARCHAR`
- `credit_unit_code VARCHAR NULL`
- `effective_from_ms BIGINT`
- `effective_to_ms BIGINT NULL`
- `status VARCHAR`
- `created_by BIGINT NULL`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

Unique constraints:

- unique `(tenant_id, organization_id, plan_code, plan_version)`

Plan types:

- `provider_cost`
- `retail_credit`
- `benefit_conversion`

#### `ai_pricing_rate`

Purpose:

- individual pricing rules

Key columns:

- `pricing_rate_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `pricing_plan_version_id BIGINT NOT NULL`
- `metric_code VARCHAR`
- `match_channel_code VARCHAR NULL`
- `match_model_code VARCHAR NULL`
- `match_provider_code VARCHAR NULL`
- `match_capability_code VARCHAR NULL`
- `charge_unit VARCHAR`
- `unit_size DECIMAL(24,8)`
- `price_value DECIMAL(24,8)`
- `rounding_mode VARCHAR`
- `minimum_charge DECIMAL(24,8) NULL`
- `sort_order INT`
- `status VARCHAR`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

### 5. Request Facts, Metrics, Charges, And Settlement

#### `ai_request_meter_fact`

Purpose:

- one request-level fact row per billable gateway request

Key columns:

- `request_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `user_id BIGINT NOT NULL`
- `account_id BIGINT NOT NULL`
- `api_key_id BIGINT NULL`
- `api_key_hash VARCHAR NULL`
- `auth_type VARCHAR`
- `jwt_subject VARCHAR NULL`
- `platform VARCHAR NULL`
- `owner VARCHAR NULL`
- `request_trace_id VARCHAR NULL`
- `gateway_request_ref VARCHAR NULL`
- `upstream_request_ref VARCHAR NULL`
- `protocol_family VARCHAR`
- `capability_code VARCHAR`
- `channel_code VARCHAR`
- `model_code VARCHAR`
- `provider_code VARCHAR`
- `request_status VARCHAR`
- `usage_capture_status VARCHAR`
- `cost_pricing_plan_version_id BIGINT NULL`
- `retail_pricing_plan_version_id BIGINT NULL`
- `estimated_credit_hold DECIMAL(24,8)`
- `actual_credit_charge DECIMAL(24,8) NULL`
- `actual_provider_cost DECIMAL(24,8) NULL`
- `started_at_ms BIGINT`
- `finished_at_ms BIGINT NULL`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

Indexes:

- `(tenant_id, organization_id, user_id, created_at_ms DESC)`
- `(tenant_id, organization_id, api_key_id, created_at_ms DESC)`
- `(tenant_id, organization_id, provider_code, model_code, created_at_ms DESC)`

#### `ai_request_meter_metric`

Purpose:

- normalized metric rows for a request

Key columns:

- `request_metric_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `request_id BIGINT NOT NULL`
- `metric_code VARCHAR`
- `quantity DECIMAL(24,8)`
- `provider_field VARCHAR NULL`
- `source_kind VARCHAR`
- `capture_stage VARCHAR`
- `is_billable BOOLEAN`
- `captured_at_ms BIGINT`

#### `ai_request_charge_line`

Purpose:

- line-level priced outcomes for a request

Key columns:

- `request_charge_line_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `request_id BIGINT NOT NULL`
- `line_kind VARCHAR`
- `metric_code VARCHAR NULL`
- `quantity DECIMAL(24,8)`
- `pricing_plan_version_id BIGINT NULL`
- `pricing_rate_id BIGINT NULL`
- `charge_amount DECIMAL(24,8)`
- `credit_amount DECIMAL(24,8)`
- `currency_code VARCHAR NULL`
- `credit_unit_code VARCHAR NULL`
- `created_at_ms BIGINT`

Line kinds:

- `provider_cost`
- `retail_charge`
- `benefit_offset`
- `manual_adjustment`

#### `ai_request_settlement`

Purpose:

- final settlement envelope for one request

Key columns:

- `request_settlement_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `request_id BIGINT NOT NULL`
- `account_id BIGINT NOT NULL`
- `user_id BIGINT NOT NULL`
- `hold_id BIGINT NULL`
- `settlement_status VARCHAR`
- `estimated_credit_hold DECIMAL(24,8)`
- `released_credit_amount DECIMAL(24,8)`
- `captured_credit_amount DECIMAL(24,8)`
- `provider_cost_amount DECIMAL(24,8)`
- `retail_charge_amount DECIMAL(24,8)`
- `shortfall_amount DECIMAL(24,8)`
- `refunded_amount DECIMAL(24,8)`
- `settled_at_ms BIGINT`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

#### `ai_request_reconciliation`

Purpose:

- late usage corrections, stream-final updates, webhook corrections, or refund-driven adjustments

Key columns:

- `request_reconciliation_id BIGINT PRIMARY KEY`
- `tenant_id BIGINT NOT NULL`
- `organization_id BIGINT NOT NULL DEFAULT 0`
- `request_id BIGINT NOT NULL`
- `reconciliation_type VARCHAR`
- `status VARCHAR`
- `delta_amount DECIMAL(24,8)`
- `delta_credit DECIMAL(24,8)`
- `reason VARCHAR`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

### 6. Commerce, Grants, And Coupons

These are required for a complete account system even if implemented after the kernel:

- `ai_commerce_product`
- `ai_commerce_product_benefit`
- `ai_commerce_order`
- `ai_commerce_order_line`
- `ai_commerce_payment`
- `ai_commerce_fulfillment`
- `ai_coupon_template`
- `ai_coupon_code`
- `ai_coupon_redemption`

All of these tables follow the same `tenant_id + organization_id` rule and issue assets into `ai_account_benefit_lot` instead of mutating balances directly.

## Request Lifecycle

### Admission

1. authenticate request by JWT or API key
2. resolve canonical gateway auth context
3. resolve user account
4. resolve provider route
5. resolve pricing plan versions
6. estimate usage and charge
7. create request fact with pending status
8. create hold and lot allocations
9. reject if hold cannot be covered

### Execution

1. dispatch upstream
2. relay response
3. capture final usage or best deterministic estimate
4. normalize metrics into request metric rows

### Settlement

1. price metric rows into charge lines
2. capture held assets
3. release unused hold
4. write settlement envelope
5. emit immutable ledger entries
6. refresh balance projection
7. update usage and reporting projections

### Reconciliation

If stream-final usage, async usage, or webhook-delayed usage arrives later:

- write reconciliation records
- never mutate the historical audit trail without evidence

## Query And Reporting Model

The system must support these first-class reporting cuts:

- by tenant
- by organization
- by user
- by API key
- by provider
- by model
- by capability
- by auth type
- by cost vs retail charge
- by meter code

Tenant reporting is mandatory even though tenant direct charging is not.

## Compatibility Strategy For The Current Repository

Existing coarse tables remain compatibility projections during migration:

- `ai_usage_records`
- `ai_billing_ledger_entries`
- `ai_billing_quota_policies`
- `ai_model_price`

Recommended long-term direction:

- `ai_usage_records` becomes a projection over `ai_request_meter_fact` and selected metrics
- `ai_billing_ledger_entries` becomes a projection over `ai_request_settlement` and `ai_account_ledger_entry`
- `ai_billing_quota_policies` is superseded by account policy and hold admission rules
- `ai_model_price` becomes a compatibility projection over `ai_pricing_plan` and `ai_pricing_rate`

## Indexing And Storage Standards

### Standard Filtering Prefix

For almost every table except `ai_tenant`, the leading operational filter prefix should be:

- `(tenant_id, organization_id, ...)`

### Time-Series Tables

For request and ledger event tables:

- include descending time indexes by the main reporting subject
- prefer immutable append-only writes

Examples:

- `(tenant_id, organization_id, user_id, created_at_ms DESC)`
- `(tenant_id, organization_id, account_id, created_at_ms DESC)`
- `(tenant_id, organization_id, api_key_id, created_at_ms DESC)`

### Financial Safety Rules

- never hard delete settlement or ledger rows
- use status transitions and reconciliation records instead of destructive rewrites
- preserve idempotency keys on gateway request settlement paths

## Error Handling And Idempotency

Required behaviors:

- repeated request-finalization attempts must be idempotent
- repeated webhook or late-usage reconciliation must be deduplicated
- holds must expire safely if upstream execution never returns
- settlement failure must not orphan opaque balance mutations

Recommended idempotency anchors:

- gateway request trace ID
- provider request reference
- internal request ID

## Verification Strategy

The design is only valid if the following verification layers pass:

- domain tests for account, hold, lot, ledger, and settlement rules
- JWT and API-key auth-context tests proving correct `tenant_id + organization_id + user_id` resolution
- storage migration tests for all canonical `ai_` tables
- gateway tests for hold creation, insufficient balance rejection, settlement capture, release, and reconciliation
- provider normalization tests for OpenAI, Anthropic, Gemini, and other major compatibility surfaces
- admin and portal contract tests for account, API key, settlement, and reporting views
- compatibility tests showing the legacy summary views still function during migration

## Implementation Order

Recommended order:

1. freeze domain and storage contracts
2. add canonical schema and projections
3. upgrade identity and auth context
4. implement account lots, holds, and ledger
5. implement row-based meters and pricing plans
6. migrate gateway execution to hold-and-settle
7. lift admin and portal onto the new kernel
8. add analytics, reconciliation, and finance export layers

## Final Recommendation

The best long-term architecture for `sdkwork-api-router` is:

- all physical tables with `ai_` prefix
- `tenant_id + organization_id` on every table except `ai_tenant`
- user-scoped payable accounts
- tenant-scoped reporting and governance
- unified JWT and API-key attribution
- lot-based assets instead of single-field balances
- hold-then-settle request lifecycle
- row-based canonical meters
- versioned pricing plans
- immutable ledger and settlement evidence
- compatibility projections during migration

Anything simpler will create another round of schema breakage once multimodal usage, packages, refunds, reconciliation, and provider diversity increase.

## References

- AWS consolidated billing: https://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/consolidated-billing.html
- AWS cost allocation tags: https://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/cost-alloc-tags.html
- AWS Cost and Usage Report: https://docs.aws.amazon.com/cur/latest/userguide/what-is-cur.html
- Azure cost management and billing overview: https://learn.microsoft.com/en-us/azure/cost-management-billing/cost-management-billing-overview
- Azure billing scopes: https://learn.microsoft.com/en-us/azure/cost-management-billing/costs/understand-work-scopes
- Google Cloud Billing export: https://cloud.google.com/billing/docs/how-to/export-data-bigquery-tables
- Google Cloud billing subaccounts reference: https://docs.cloud.google.com/billing/docs/reference/rest/v1/billingAccounts.subAccounts
- Alibaba Cloud resource planning and cost allocation: https://help.aliyun.com/zh/caf/resource-planning
- Alibaba Cloud billing FAQ: https://help.aliyun.com/zh/user-center/support/billing-faqs
- Tencent Cloud cost allocation tags: https://cloud.tencent.com/document/product/555/37959
- Tencent Cloud cost analysis: https://cloud.tencent.com/document/product/555/60369
- FOCUS specification: https://focus.finops.org/focus-specification/
