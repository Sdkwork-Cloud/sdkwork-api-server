# API Router Billing And Metering Architecture Design

## Goal

Rebuild `sdkwork-api-router` around a production-grade metering, pricing, wallet, and settlement architecture that can:

- preserve protocol compatibility for OpenAI, Claude Code via Anthropic Messages, and Google Gemini
- normalize heterogeneous token usage into one canonical meter model
- charge by real usage instead of coarse `units`
- deduct credits from project accounts on every billable request
- support cash recharge, coupon grants, and token or request packages
- separate upstream provider cost from customer-facing retail billing
- keep `sdkwork-router-admin` as the super-admin control plane
- keep `sdkwork-router-portal` as the customer-facing self-service product

This design replaces the current summary-oriented billing model with a real settlement kernel.

## Why The Current Architecture Is Not Enough

The repository already has good building blocks:

- protocol compatibility for OpenAI, Anthropic Messages, and Gemini GenerateContent
- channel and proxy-provider catalog tables
- model pricing rows
- usage records and billing summary APIs
- portal surfaces for credits, billing, and recharge-oriented UX

The current implementation is still structurally incomplete for real billing:

- `usage` is stored as one coarse row with `units`, `amount`, `input_tokens`, `output_tokens`, and `total_tokens`
- `billing` is a simple ledger with `project_id`, `units`, and `amount`
- quota admission is based on fixed estimated `requested_units`
- there is no immutable request settlement record
- there is no hold or pre-authorization workflow
- there is no credit wallet or lot-based balance engine
- there is no coupon redemption ledger or commerce fulfillment model
- provider cost pricing and customer sell pricing are not separated
- pricing is not versioned, so later audits cannot reproduce historic charges exactly

That is acceptable for demos, but not for a long-lived API router product.

## External Compatibility Findings

The billing kernel must treat upstream usage as heterogeneous, not uniform.

### OpenAI

OpenAI usage and pricing distinguish multiple token dimensions:

- input tokens
- cached input tokens
- output tokens
- reasoning tokens in some model families

OpenAI pricing also distinguishes at least input, cached input, and output classes, and prompt-caching guidance explicitly exposes cached-token accounting.

### Anthropic And Claude Code

Anthropic Messages usage and pricing distinguish:

- `input_tokens`
- `output_tokens`
- `cache_creation_input_tokens`
- `cache_read_input_tokens`

Claude Code gateway compatibility rides on the Anthropic Messages surface and must preserve Anthropic request semantics and headers while still routing through the shared metering kernel.

### Gemini

Gemini `usageMetadata` includes more than plain prompt and output counts. Relevant fields include:

- `promptTokenCount`
- `candidatesTokenCount`
- `totalTokenCount`
- `cachedContentTokenCount`
- `thoughtsTokenCount`
- `toolUsePromptTokenCount`

Gemini therefore cannot be modeled correctly with only input and output token columns.

## Non-Negotiable Design Decisions

### 1. Proxy Provider Is The Routing And Billing Execution Unit

`channel` remains the upstream ecosystem or model-provider family, such as `openai`, `anthropic`, `gemini`, `openrouter`, or `ollama`.

`proxy_provider` is the concrete upstream route target configured by operators. Examples:

- OpenAI official
- Azure OpenAI production
- OpenRouter primary
- Anthropic official
- Gemini official
- enterprise reverse proxy

All routing, credentials, upstream cost pricing, health, and execution policy bind to `proxy_provider`.

### 2. Pricing Is Versioned And Snapshot-Based

The router must never compute charges from mutable current prices after a request finishes.

Every request settlement must snapshot:

- the selected proxy provider
- the resolved channel and model
- the upstream cost price plan version
- the retail price plan version
- the normalized meter quantities used for settlement

This is required for replay, audits, refunds, and disputes.

### 3. Strong Consistency With Pre-Authorization Is The Default

The default account policy is:

1. estimate worst-case billable usage before dispatch
2. create a hold against the account
3. reject the request if the hold cannot be covered
4. execute the request
5. settle the actual charge
6. release unused held credits

Negative balances are disabled by default.

Enterprise postpaid or overdraft behavior is an explicit account-policy exception, not the default path.

### 4. Benefits Are Lot-Based, Not Just Balance-Based

The system must not treat every recharge source as a flat numeric balance.

Instead, every credit or allowance grant creates a benefit lot. A lot may represent:

- general-purpose credits
- request-count allowance
- token allowance
- scoped meter allowance for a channel, model, or proxy provider

This makes coupons, traffic packages, recharge gifts, and manual grants all first-class and auditable.

### 5. Upstream Cost And Customer Billing Are Separate Price Plan Families

The router needs two different pricing planes:

- upstream cost pricing: what the operator pays the proxy provider
- retail billing pricing: what the portal customer pays in credits

They must not share the same table semantics, because profitability, promotions, and subsidies require separation.

## Bounded Subprojects

This architecture spans four bounded subsystems:

1. metering and settlement kernel
2. wallet, coupon, recharge, and package commerce
3. admin control plane for provider and price management
4. portal control plane for balances, orders, redemption, and request billing views

The first critical subproject is the metering and settlement kernel. All other work depends on it.

## Canonical Metering Model

### Meter Definitions

Introduce canonical meter codes instead of hard-coding only `input` and `output`.

The initial standard meter catalog should include:

- `request.count`
- `token.input`
- `token.output`
- `token.cache_read`
- `token.cache_write`
- `token.reasoning`
- `token.tool_input`
- `token.thought`

The catalog must be extensible so future APIs can add:

- audio seconds
- image generations
- video seconds
- storage
- session minutes

### Request Meter Fact

Each billable gateway request gets exactly one immutable request fact row.

The fact row records:

- request identity
- auth identity
- tenant and project scope
- protocol family such as `openai`, `anthropic_messages`, or `gemini_generate_content`
- capability such as `chat`, `responses`, `embeddings`, `audio`, or `images`
- route key and selected proxy provider
- upstream request id when available
- request lifecycle status
- whether usage is actual, estimated, reconciled, or refunded
- pricing plan version ids used for settlement
- final monetary and credit totals

### Request Meter Metric Rows

Each request fact can have many metric rows.

Each metric row records:

- `request_id`
- `metric_code`
- `quantity`
- `source_kind`
- `provider_field`
- `is_billable`
- `capture_stage`

This row-based model supports OpenAI, Anthropic, and Gemini without schema churn every time a provider adds a new usage field.

### Provider Usage Normalization Rules

Normalize provider-specific usage as follows:

- OpenAI `prompt_tokens` -> `token.input`
- OpenAI `prompt_tokens_details.cached_tokens` -> `token.cache_read`
- OpenAI `completion_tokens` -> `token.output`
- OpenAI `completion_tokens_details.reasoning_tokens` -> `token.reasoning`
- Anthropic `input_tokens` -> `token.input`
- Anthropic `cache_creation_input_tokens` -> `token.cache_write`
- Anthropic `cache_read_input_tokens` -> `token.cache_read`
- Anthropic `output_tokens` -> `token.output`
- Gemini `promptTokenCount` -> `token.input`
- Gemini `cachedContentTokenCount` -> `token.cache_read`
- Gemini `candidatesTokenCount` -> `token.output`
- Gemini `thoughtsTokenCount` -> `token.thought`
- Gemini `toolUsePromptTokenCount` -> `token.tool_input`

When a provider does not return final usage:

- use a deterministic estimate profile for admission
- settle as `estimated` if the provider never returns usage
- allow later reconciliation if a delayed result or webhook arrives

## Pricing Architecture

### Price Plan Families

Introduce versioned price plans with distinct plan types:

- `provider_cost`
- `retail_credit`
- `benefit_conversion`

### Price Plan Scope

A price plan may target:

- all traffic
- one channel
- one model
- one proxy provider
- one tenant
- one project

Matching precedence:

1. project + proxy provider + model
2. tenant + proxy provider + model
3. proxy provider + model
4. channel + model
5. channel
6. global default

### Pricing Rate Rows

Each price plan version contains many rate rows.

Each rate row includes:

- `metric_code`
- `charge_unit`
- `unit_size`
- `price_value`
- `currency_code` or `credit_unit_code`
- `rounding_mode`
- `minimum_charge`
- optional matcher scope such as model, channel, or provider

This architecture removes the need to keep extending `ai_model_price` with more special-purpose columns.

### Relationship To Existing `ai_model_price`

The current `ai_model_price` table should be treated as a compatibility read model only.

The long-term source of truth becomes:

- `ai_proxy_provider_model` for model availability and route metadata
- `ai_pricing_plan` plus `ai_pricing_rate` for actual billing and cost rules

`ai_model_price` can be retained as a synthesized projection for existing admin screens and legacy consumers during migration.

## Account, Credits, And Benefits

### Account Scope

The primary billable account is project-scoped.

Recommended hierarchy:

- tenant account for top-level finance and funding
- project account for request settlement

Project accounts may inherit funding from tenant-level grants, but request settlement writes against the project account.

### Benefit Lot Types

Each granted lot has a `benefit_type`:

- `credit`
- `request_allowance`
- `meter_allowance`

Examples:

- cash recharge creates a `credit` lot
- welcome coupon may create a `credit` lot
- one hundred free calls coupon creates a `request_allowance` lot
- GPT-4 package with 1 million input tokens creates a `meter_allowance` lot scoped to selected metrics and models

### Lot Matching Policy

Default spend policy:

1. earliest expiry first
2. narrower scope first
3. promotional or allowance lots before cash lots
4. lower acquisition cost first when priorities still tie

Refunds must reverse the original allocation chain.

### Holds And Settlement

Admission creates a hold with allocation rows against specific lots.

Settlement consumes the hold and emits:

- one immutable request settlement row
- one or more charge lines
- one or more account ledger entries
- allocation rows showing which lots funded which charge lines

If actual charge is lower than estimated:

- release the unused hold

If actual charge exceeds the estimate:

- attempt incremental capture
- reject or mark shortfall according to account policy

## Coupon, Recharge, And Package Commerce

### Products

Treat commerce as product fulfillment into benefit lots.

Required product types:

- `cash_topup`
- `credit_pack`
- `meter_pack`
- `subscription_plan`
- `manual_grant`

### Coupons

Coupons must be modeled as grants, not discounts only.

Required coupon grant modes:

- `credit_grant`
- `request_allowance_grant`
- `meter_allowance_grant`
- `order_discount`

Coupon redemption records must keep:

- coupon code
- template and campaign
- redeemed account
- resulting order or grant
- idempotency key
- expiration evidence

### Recharge Flow

Recharge and purchases should follow:

1. create order
2. create payment attempt
3. mark payment success
4. create fulfillment record
5. issue benefit lots
6. write account ledger issue entries

No payment success should ever update balances directly without an immutable fulfillment and ledger record.

## Canonical Database Design

### Catalog And Routing

#### `ai_channel`

Canonical channel registry.

Key columns:

- `channel_id`
- `channel_name`
- `channel_kind`
- `sort_order`
- `is_builtin`
- `is_active`
- `created_at_ms`
- `updated_at_ms`

#### `ai_proxy_provider`

Concrete upstream execution targets.

Key columns:

- `proxy_provider_id`
- `primary_channel_id`
- `provider_type`
- `adapter_kind`
- `base_url`
- `display_name`
- `billing_currency_code`
- `usage_protocol_family`
- `supports_stream_usage`
- `supports_async_reconcile`
- `supports_prompt_caching`
- `settlement_mode`
- `is_active`
- `created_at_ms`
- `updated_at_ms`

#### `ai_proxy_provider_channel`

Multi-channel binding for one proxy provider.

#### `ai_model`

Channel model catalog.

#### `ai_proxy_provider_model`

Provider-specific model offering and execution metadata.

Key columns:

- `proxy_provider_id`
- `channel_id`
- `model_id`
- `provider_model_id`
- `provider_model_family`
- `capabilities_json`
- `context_window`
- `max_output_tokens`
- `supports_prompt_caching`
- `supports_reasoning_usage`
- `supports_tool_usage_metrics`
- `is_default_route`
- `is_active`
- `created_at_ms`
- `updated_at_ms`

This table becomes the routing catalog for provider-model availability.

### Metering And Pricing

#### `ai_meter_definition`

Canonical meter catalog.

Key columns:

- `metric_code`
- `metric_name`
- `unit_kind`
- `aggregation_kind`
- `default_rounding_mode`
- `is_active`

#### `ai_pricing_plan`

Versioned pricing plan header.

Key columns:

- `plan_id`
- `plan_version`
- `plan_type`
- `status`
- `scope_kind`
- `scope_id`
- `display_name`
- `effective_from_ms`
- `effective_to_ms`
- `created_by`
- `created_at_ms`

Primary key recommendation:

- surrogate `pricing_plan_version_id`
- unique constraint on `plan_id + plan_version`

#### `ai_pricing_rate`

Rate rows for a pricing plan version.

Key columns:

- `pricing_plan_version_id`
- `metric_code`
- `match_channel_id`
- `match_model_id`
- `match_proxy_provider_id`
- `charge_unit`
- `unit_size`
- `price_value`
- `rounding_mode`
- `minimum_charge`
- `sort_order`

#### `ai_request_meter_fact`

One immutable row per request.

Key columns:

- `request_id`
- `request_trace_id`
- `gateway_request_id`
- `tenant_id`
- `project_id`
- `account_id`
- `api_key_hash`
- `protocol_family`
- `capability`
- `channel_id`
- `model_id`
- `route_key`
- `proxy_provider_id`
- `provider_request_id`
- `request_status`
- `usage_capture_status`
- `cost_pricing_plan_version_id`
- `retail_pricing_plan_version_id`
- `estimated_credit_hold`
- `actual_credit_charge`
- `actual_provider_cost`
- `settled_at_ms`
- `created_at_ms`
- `updated_at_ms`

#### `ai_request_meter_metric`

Per-request normalized usage metrics.

Primary key recommendation:

- surrogate `request_metric_id`

Important columns:

- `request_id`
- `metric_code`
- `quantity`
- `provider_field`
- `source_kind`
- `capture_stage`
- `captured_at_ms`

#### `ai_request_charge_line`

One row per priced metric per request.

Important columns:

- `request_id`
- `line_kind` such as `provider_cost`, `retail_charge`, or `benefit_offset`
- `metric_code`
- `quantity`
- `pricing_plan_version_id`
- `applied_rate_id`
- `charge_amount`
- `credit_amount`

#### `ai_request_settlement`

Final request settlement envelope.

Important columns:

- `request_id`
- `account_id`
- `settlement_status`
- `estimated_credit_hold`
- `released_credit_amount`
- `captured_credit_amount`
- `provider_cost_amount`
- `retail_charge_amount`
- `shortfall_amount`
- `refunded_amount`
- `settled_at_ms`

### Wallet And Ledger

#### `ai_billing_account`

Project or tenant wallet account.

Key columns:

- `account_id`
- `account_scope_type`
- `account_scope_id`
- `parent_account_id`
- `currency_code`
- `credit_unit_code`
- `allow_overdraft`
- `overdraft_limit`
- `status`
- `created_at_ms`
- `updated_at_ms`

#### `ai_account_benefit_lot`

Issued credits or allowances.

Key columns:

- `lot_id`
- `account_id`
- `benefit_type`
- `source_type`
- `source_id`
- `grant_reason`
- `scope_json`
- `original_quantity`
- `remaining_quantity`
- `priority`
- `acquired_unit_cost`
- `expires_at_ms`
- `issued_at_ms`
- `active`

#### `ai_account_hold`

Request pre-authorization header.

Key columns:

- `hold_id`
- `request_id`
- `account_id`
- `hold_status`
- `estimated_quantity`
- `captured_quantity`
- `released_quantity`
- `expires_at_ms`
- `created_at_ms`
- `updated_at_ms`

#### `ai_account_hold_allocation`

Lot-level reservation rows under a hold.

#### `ai_account_ledger_entry`

Immutable account event ledger.

Entry types:

- `issue`
- `hold_create`
- `hold_release`
- `settle_capture`
- `settle_refund`
- `expire`
- `adjustment`
- `transfer_in`
- `transfer_out`

Important columns:

- `entry_id`
- `account_id`
- `request_id`
- `hold_id`
- `entry_type`
- `benefit_type`
- `quantity_delta`
- `balance_after`
- `source_type`
- `source_id`
- `created_at_ms`

#### `ai_account_ledger_allocation`

Maps a ledger entry to one or more lots.

### Commerce And Coupon Tables

#### `ai_commerce_product`

Catalog of recharge packs, token packs, subscriptions, and grants.

#### `ai_commerce_product_benefit`

Defines what fulfillment grants for each SKU.

Examples:

- 100 credits
- 1,000 request allowance
- 1,000,000 `token.input` allowance for channel `openai` model family `gpt-4.1`

#### `ai_commerce_order`

Commercial order envelope.

#### `ai_commerce_order_line`

SKU lines.

#### `ai_commerce_payment`

Payment intent or payment record.

#### `ai_commerce_fulfillment`

Order fulfillment to benefit lots.

#### `ai_coupon_template`

Reusable coupon rule template.

#### `ai_coupon_code`

Issued redemption code.

#### `ai_coupon_redemption`

Immutable redemption record.

### Compatibility Views And Deprecation Path

The following existing tables should become compatibility projections:

- `ai_usage_records`
- `ai_billing_ledger_entries`
- `ai_billing_quota_policies`
- `ai_model_price`

Recommended long-term treatment:

- `ai_usage_records` becomes a view over `ai_request_meter_fact` plus selected common metrics
- `ai_billing_ledger_entries` becomes a view over `ai_request_settlement` or `ai_account_ledger_entry`
- `ai_billing_quota_policies` is gradually replaced by account policy and admission policy tables
- `ai_model_price` becomes a synthesized compatibility surface over `ai_pricing_rate`

## Request Lifecycle

### Admission

1. authenticate API key
2. resolve tenant, project, and billing account
3. resolve candidate route and selected `proxy_provider`
4. resolve provider-model offering
5. resolve applicable provider-cost and retail price plan versions
6. estimate worst-case charge from request payload, model defaults, and configured safety margins
7. create request fact in `pending_admission`
8. create account hold
9. if hold fails, reject with insufficient balance

### Execution

1. dispatch request upstream
2. stream or relay response
3. capture final usage payload or local estimate
4. normalize metrics into `ai_request_meter_metric`

### Settlement

1. price normalized metrics into provider-cost and retail charge lines
2. capture held balances against lots
3. release unused hold
4. persist request settlement
5. update compatibility projections
6. emit outbox event for summaries and analytics

### Reconciliation

Late or corrected usage from async jobs, streams, or webhooks must create a reconciliation settlement, not mutate the original request metrics in place without audit evidence.

## Admin Responsibilities

`sdkwork-router-admin` becomes the source of truth for:

- channel registry
- proxy provider registry
- proxy provider channel bindings
- provider model offerings
- upstream credential inventory
- provider-cost price plans
- retail price plans
- commerce products
- coupon templates and issued codes
- billing account policy
- request settlement audit and reconciliation

The existing routing page should stop pretending to own provider pricing. Provider pricing belongs under proxy-provider governance.

## Portal Responsibilities

`sdkwork-router-portal` becomes the source of truth for customer-facing read models and self-service actions:

- account balance and available benefits
- request billing history
- settlement details
- recharge orders
- token-package purchases
- coupon redemption
- current retail price exposure

Portal should never own provider-cost configuration.

## Migration Strategy

### Phase 1

- add new metering, pricing, account, and commerce tables
- keep current tables and APIs alive through compatibility projections
- begin dual-write for request usage and billing

### Phase 2

- move gateway admission from coarse quota to real account holds
- move admin and portal summaries to the new tables

### Phase 3

- retire direct writes to legacy `ai_usage_records` and `ai_billing_ledger_entries`
- keep them as read-only compatibility views until all clients migrate

## Verification Strategy

Required verification layers:

- provider compatibility tests for OpenAI, Anthropic Messages, Gemini GenerateContent, and Claude Code gateway flows
- request metering normalization tests for each provider usage payload shape
- price-plan precedence tests
- hold and settlement transaction tests
- idempotency tests for retries and duplicate webhooks
- refund and reconciliation tests
- coupon redemption tests for credits and request-count grants
- portal and admin contract tests for account, recharge, package, and request-billing views

## Source References

Primary external references used to shape this design:

- OpenAI prompt caching and pricing: `https://openai.com/api/pricing/`
- OpenAI prompt caching guide: `https://platform.openai.com/docs/guides/prompt-caching`
- Anthropic pricing and prompt caching: `https://docs.anthropic.com/en/docs/about-claude/pricing`
- Anthropic Messages API reference: `https://docs.anthropic.com/en/api/messages`
- Claude Code LLM gateway: `https://docs.anthropic.com/en/docs/claude-code/llm-gateway`
- Gemini API generate content and usage metadata: `https://ai.google.dev/api/generate-content`
- Gemini API pricing: `https://ai.google.dev/pricing`

## Recommendation

The correct long-term standard for `sdkwork-api-router` is:

- proxy-provider-first routing
- row-based canonical meters
- versioned pricing plans
- strong-consistency account holds
- immutable request settlement
- benefit-lot wallet accounting
- compatibility projections for legacy summary APIs

Anything less will keep the router in a demo-grade billing architecture.
