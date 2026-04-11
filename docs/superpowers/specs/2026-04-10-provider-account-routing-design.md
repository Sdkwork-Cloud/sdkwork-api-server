# Provider Account Routing Design

**Context**

The current router can select a `provider`, but not a concrete provider-owned account or endpoint under that provider. That leaves several operational gaps:

- one proxy provider cannot expose multiple routed upstream accounts cleanly
- one official provider cannot express multiple cloud accounts, API keys, or regional accounts
- pricing, health, latency, and success posture are tracked at provider scope, not account scope
- routing can prefer a provider, but execution still collapses to a single provider credential path
- admin can manage providers and provider-models, but not the executable account inventory that actually carries traffic

The existing runtime already contains useful primitives:

- `provider` is the canonical executable upstream surface
- `provider-model` is the provider-supported subset of the inventor catalog
- `credential` can already hold multiple `(tenant_id, provider_id, key_reference)` entries
- `extension-instance` already carries `credential_ref`, `base_url`, and `config`
- provider health snapshots already support `instance_id`

What is missing is a first-class account layer that binds these capabilities together for routing and admin.

## Goals

- Add a first-class `provider-account` layer under each provider.
- Allow one provider to expose multiple executable accounts:
  - API keys
  - cloud accounts
  - regional commercial accounts
  - contract or reseller accounts
- Let routing choose between:
  - providers across the market
  - accounts within one selected provider
- Support account-aware routing objectives:
  - cost priority
  - latency priority
  - success-rate priority
  - performance priority
  - availability priority
  - balanced blended objective
- Keep the data model idempotent, additive, and bootstrap-friendly under `/data`.
- Keep runtime execution, secrets, routing, and admin boundaries high-cohesion and low-coupling.

## Non-Goals For The First Delivery Slice

- Rewriting the entire routing engine so compiled candidate sets are account-native end-to-end in one change.
- Shipping a full telemetry warehouse before account-aware routing can start.
- Replacing existing provider or provider-model tables.

## Canonical Model

The catalog and execution model becomes:

1. `channel`
2. `channel-model`
3. `provider`
4. `provider-model`
5. `provider-account`
6. optional `provider-account-model`
7. optional `provider-account-price`

### Existing layers stay unchanged

- `channel` is still the inventor/vendor surface.
- `channel-model` is still the canonical publication under a channel.
- `provider` is still the upstream platform or marketplace entry.
- `provider-model` is still the provider-supported subset mapping for a canonical model.

### New layer: `provider-account`

`provider-account` is the account-scoped execution posture under one provider.

It owns:

- account identity
- account kind
- execution binding
- region / residency posture
- routing priority and weights
- operator cost and latency hints
- traffic enablement
- failure domain isolation

Recommended fields:

- `provider_account_id`
- `provider_id`
- `display_name`
- `account_kind`
  - `api_key`
  - `cloud_account`
  - `service_account`
  - `reseller_contract`
- `owner_scope`
  - `platform`
  - `tenant`
- `owner_tenant_id`
- `execution_instance_id`
- `base_url_override`
- `region`
- `priority`
- `weight`
- `enabled`
- `routing_tags`
- `health_score_hint`
- `latency_ms_hint`
- `cost_hint`
- `success_rate_hint`
- `throughput_hint`
- `max_concurrency`
- `daily_budget`
- `notes`

### Optional layer: `provider-account-model`

Different accounts under the same provider may not expose the same model set. `provider-account-model` binds one `provider-account` to a supported `provider-model`.

This allows:

- proxy account A supports only frontier models
- proxy account B supports only cheap inference pool models
- regional account C only serves China-approved models

### Optional layer: `provider-account-price`

`model-price` remains the provider-scoped public/reference price.

`provider-account-price` becomes the account-scoped override for:

- contract discounts
- reseller markup
- region-specific price
- private marketplace agreement
- infra showback for local execution pools

Resolution order should be:

1. `provider-account-price`
2. `model-price`
3. official/reference fallback

## Execution Binding

`provider-account` should not duplicate secret storage itself.

It binds to execution using the already-existing runtime layer:

- `provider-account.execution_instance_id -> extension-instance.instance_id`
- `extension-instance.credential_ref` remains the secret key reference
- `extension-instance.base_url` remains the endpoint override
- `extension-instance.config` remains the runtime-specific config bag

This keeps responsibilities clean:

- catalog owns account inventory and routing metadata
- runtime owns executable adapter/instance state
- credential store owns secrets

## Routing Architecture

### Selection phases

Routing should evolve to two phases:

1. `provider selection`
  - choose across providers according to policy/profile
2. `provider-account selection`
  - choose the account under the selected provider

This preserves compatibility while introducing account-aware routing incrementally.

### Why two phases first

A fully account-native candidate graph is the eventual target, but a two-phase rollout is the safest first step:

- existing provider policy/profile config keeps working
- compiled routing snapshots do not need a breaking redesign immediately
- gateway execution can start honoring multiple accounts per provider right away
- account-level health and contract pricing can be introduced without destabilizing provider-level policy

## Strategy Vocabulary

The current UI contains legacy strings like `priority`, `balanced`, `latency_optimized`, and `cost_optimized`, while backend routing actually supports:

- `deterministic_priority`
- `weighted_random`
- `slo_aware`
- `geo_affinity`

That mismatch should be corrected.

### Recommended model

Split routing into two concepts:

- `strategy_kind`
  - `deterministic_priority`
  - `weighted_random`
  - `geo_affinity`
  - `slo_aware`
  - `score_based`
- `objective_kind`
  - `balanced`
  - `cost`
  - `latency`
  - `success_rate`
  - `performance`
  - `availability`
  - `custom`

`strategy_kind` answers how candidates are selected.

`objective_kind` answers what the router is trying to optimize.

### Score-based routing

`score_based` should support a normalized weighted score across:

- price
- observed latency
- health
- success rate
- availability
- throughput
- region match
- configured priority/weight

Recommended score inputs:

- `score_weights.price`
- `score_weights.latency`
- `score_weights.success_rate`
- `score_weights.performance`
- `score_weights.availability`
- `score_weights.region`
- `score_weights.priority`

## Account Selection Rules

When a provider is selected:

1. filter to enabled accounts
2. filter to accounts bound to the requested model if account-model bindings exist
3. filter by healthy/available execution instances if required
4. apply region preference
5. apply selected objective
6. if tied:
  - prefer higher account priority
  - then better health
  - then lower latency
  - then lower cost
  - then stable lexical id

Failover order should be:

1. same provider, different eligible account
2. next provider candidate from routing policy

This reduces protocol drift and commercial variance during failover.

## Observability And Feedback

Account-aware routing becomes useful only if account outcomes are observable.

New or extended observability records should capture:

- `provider_account_id`
- `execution_instance_id`
- request success/failure
- latency
- retry count
- timeout / 429 / 5xx ratios
- token throughput
- observed cost basis

Provider health snapshots already support `instance_id`; that should be used as the bridge between runtime status and `provider-account`.

## Admin Surface

Admin needs a new management surface under providers:

- provider detail -> `Accounts`
- create/edit/delete provider account
- bind execution instance
- set region, priority, weight, and objective hints
- view price coverage
- view supported provider-model subset
- enable or disable account

Provider-model management remains on the provider.

Account-level model enablement belongs to `provider-account-model`.

Routing admin should also be normalized:

- replace legacy freeform strategy labels with canonical enums
- expose `objective_kind`
- expose score weights
- expose same-provider failover preference

## API Surface

Recommended new admin endpoints:

- `GET /admin/provider-accounts`
- `POST /admin/provider-accounts`
- `DELETE /admin/provider-accounts/{provider_account_id}`
- `GET /admin/provider-account-models`
- `POST /admin/provider-account-models`
- `DELETE /admin/provider-account-models/{provider_account_id}/channels/{channel_id}/models/{model_id}`
- `GET /admin/provider-account-prices`
- `POST /admin/provider-account-prices`
- `DELETE /admin/provider-account-prices/{provider_account_id}/channels/{channel_id}/models/{model_id}`

## Bootstrap Data Layout

Additive repository-backed seed layout should extend `/data` with:

- `data/provider-accounts/*.json`
- `data/provider-account-models/*.json`
- `data/provider-account-prices/*.json`

Importer order should become:

1. providers
2. official provider configs
3. provider accounts
4. models
5. channel models
6. provider models
7. provider account models
8. model prices
9. provider account prices
10. routing

This keeps references safe:

- account cannot exist without provider
- account-model cannot exist without provider-account and provider-model
- account-price cannot exist without provider-account and routable model support

## Compatibility And Migration

Migration should be additive.

### Existing providers

Existing providers continue to work even if no `provider-account` rows exist.

Compatibility behavior:

- if provider accounts exist, provider execution resolves through account selection
- if provider accounts do not exist, fall back to current provider-level behavior

### Existing extension instances

Bootstrap and runtime can initially map one default account to one existing extension instance:

- `provider-openai-official -> acct-openai-default -> instance provider-openai-official`
- `provider-openrouter-main -> acct-openrouter-default -> instance provider-openrouter-main`

This preserves today’s execution posture while unlocking future multiple-account expansion.

### Existing pricing

Existing `model-price` rows remain valid and become the provider-scoped fallback reference price.

## First Delivery Slice

The first slice should deliver:

- `provider-account` domain model
- storage and migrations
- admin CRUD
- bootstrap `/data/provider-accounts`
- gateway account resolution for a selected provider
- canonical routing strategy normalization in admin

The first slice intentionally does not require:

- full account-native compiled routing snapshots
- full account-native decision logs
- provider-account-model or provider-account-price overrides

## Later Slices

After the first slice:

1. add `provider-account-model`
2. add `provider-account-price`
3. promote compiled routing snapshots and decision logs to account-aware structures
4. add `score_based` strategy and objective-driven scoring
5. expose account telemetry and route explainability in admin

## Testing

Required coverage should include:

- storage roundtrip for provider accounts
- bootstrap importer idempotency with provider accounts
- gateway selects default provider account when multiple are present
- gateway fails over to a second account before crossing providers
- disabled account is ignored
- region-preferred account wins when eligible
- admin strategy values serialize to canonical backend enums

## Recommendation

Implement `provider-account` first as the missing execution-routing abstraction, while reusing `extension-instance` and `credential_ref` for secrets and endpoint binding. That produces immediate commercial value without forcing a risky one-shot rewrite of the routing engine.
