# AI Router Schema Standardization Design

## Goal

Standardize the sdkwork-api-router control-plane schema around canonical `ai_*` table names, preserve backward compatibility for legacy SQL callers, add first-class channel/model/provider pricing management, and extend the standalone admin application so operators can manage channels, models, model pricing, proxy providers, router credentials, and app API keys from one coherent catalog workflow.

## Current State

- Canonical data is currently spread across mixed table families such as `identity_*`, `tenant_*`, `catalog_*`, `routing_*`, `billing_*`, and `extension_*`.
- Runtime-facing model records are provider-centric (`external_name + provider_id`) while the product requirement is channel-centric (`channel -> models -> provider pricing`).
- Proxy providers expose only partial model coverage, but the old mental model still leans toward "provider implies whole channel," which is incorrect for route config, pricing, and admin governance.
- Gateway app API keys only persist hashed values in storage, while the product requirement now asks for persisted `raw_key` plus `hashed_key`.
- Router upstream credentials are stored in `credential_records`; the product requirement wants the router config secret registry to live in `ai_router_credential_records`.
- The admin catalog page exposes channel/provider/credential/model CRUD, but it does not support channel-scoped model management or provider-aware pricing CRUD.
- The commercial account kernel exists at the storage/runtime layer, but repository bootstrap did not yet treat account balances, metering, settlements, and reconciliation as first-class seed domains.

## Canonical Schema Standard

### Naming Rules

- Every physical table uses the `ai_` prefix.
- Every column uses lowercase snake_case.
- IDs remain textual and stable across SQLite/Postgres.
- Compatibility with legacy names is preserved through migration copy steps plus compatibility views where direct SQL callers still expect old names.

### Canonical Tables

- `ai_portal_users`
- `ai_admin_users`
- `ai_tenants`
- `ai_projects`
- `ai_coupon_campaigns`
- `ai_channel`
- `ai_proxy_provider`
- `ai_proxy_provider_channel`
- `ai_router_credential_records`
- `ai_model`
- `ai_proxy_provider_model`
- `ai_model_price`
- `ai_routing_policies`
- `ai_routing_policy_provider`
- `ai_project_routing_preferences`
- `ai_routing_decision_logs`
- `ai_provider_health_records`
- `ai_usage_records`
- `ai_billing_ledger_entries`
- `ai_billing_quota_policies`
- `ai_account`
- `ai_account_benefit_lot`
- `ai_account_hold`
- `ai_account_hold_allocation`
- `ai_account_ledger_entry`
- `ai_account_ledger_allocation`
- `ai_request_meter_fact`
- `ai_request_meter_metric`
- `ai_request_settlement`
- `ai_account_commerce_reconciliation_state`
- `ai_pricing_plan`
- `ai_pricing_rate`
- `ai_app_api_keys`
- `ai_extension_installations`
- `ai_extension_instances`
- `ai_service_runtime_nodes`
- `ai_extension_runtime_rollouts`
- `ai_extension_runtime_rollout_participants`
- `ai_standalone_config_rollouts`
- `ai_standalone_config_rollout_participants`

## Catalog Data Model

### Channel

`ai_channel` is the canonical registry of API surfaces exposed by the router. It stores:

- `channel_id`
- `channel_name`
- `channel_description`
- `sort_order`
- `is_builtin`
- `is_active`
- `created_at_ms`
- `updated_at_ms`

Default seed rows:

- `openai`
- `anthropic`
- `gemini`
- `openrouter`
- `ollama`

### Proxy Provider

`ai_proxy_provider` stores upstream proxy providers and their runtime metadata. A provider may attach to multiple channels through `ai_proxy_provider_channel`, while `primary_channel_id` remains the default channel.

For shipped bootstrap data, a proxy-provider channel binding is treated as an execution promise, not just metadata. If a provider is bound to a channel in repository seed data, that seed set should also include:

- at least one `ai_proxy_provider_model` row for that `(provider, channel)`
- at least one `ai_model_price` row for the same canonical model/provider combination
- at least one usable route profile or policy path when the binding is meant to be operator-facing by default

### Channel Model

`ai_model` is the channel-model registry. A row defines that a model belongs to a channel regardless of provider pricing availability. It stores:

- `channel_id`
- `model_id`
- `model_display_name`
- `capabilities_json`
- `streaming_enabled`
- `context_window`
- `description`
- `created_at_ms`
- `updated_at_ms`

Primary key: `(channel_id, model_id)`.

### Model Pricing

`ai_model_price` is the provider-specific routing price registry. It defines whether a provider offers a given channel model and what the pricing posture is. It stores:

- `channel_id`
- `model_id`
- `proxy_provider_id`
- `currency_code`
- `price_unit`
- `input_price`
- `output_price`
- `cache_read_price`
- `cache_write_price`
- `request_price`
- `is_active`
- `created_at_ms`
- `updated_at_ms`

Primary key: `(channel_id, model_id, proxy_provider_id)`.

This table becomes the canonical provider-model availability source for routing and `/v1/models`.

### Commercial Readiness Layer

The catalog price surface is not enough on its own. Commercial readiness also needs an account kernel that can represent:

- who pays
- what balance is available
- which benefit lots were consumed
- what was held before execution
- what was actually charged after execution
- how request cost and retail charge were measured
- how account state reconciles back to commerce orders and projects

The canonical account/billing tables therefore split cleanly into two concerns:

- catalog-facing provider price posture
  - `ai_model_price`
  - operator-visible official/proxy/local pricing
- account-facing commercial execution posture
  - `ai_account`
  - `ai_account_benefit_lot`
  - `ai_account_hold`
  - `ai_account_hold_allocation`
  - `ai_account_ledger_entry`
  - `ai_account_ledger_allocation`
  - `ai_request_meter_fact`
  - `ai_request_meter_metric`
  - `ai_request_settlement`
  - `ai_account_commerce_reconciliation_state`
  - `ai_pricing_plan`
  - `ai_pricing_rate`

This separation is intentional:

- `channel` still means the canonical inventor/vendor
- `provider` still means the executable official/proxy/local endpoint
- `route config` still chooses providers
- `ai_model_price` describes provider catalog pricing
- pricing plans/rates describe internal billing and cost policy
- request metering and settlement bridge the catalog decision back into balance, ledger, and reconciliation state

## Credentials And App Keys

### Router Credentials

`ai_router_credential_records` replaces `credential_records` as the canonical router config secret registry. It keeps encrypted storage metadata:

- `tenant_id`
- `proxy_provider_id`
- `key_reference`
- `secret_backend`
- `secret_local_file`
- `secret_keyring_service`
- `secret_master_key_id`
- `secret_ciphertext`
- `secret_key_version`
- `created_at_ms`
- `updated_at_ms`

### App API Keys

`ai_app_api_keys` replaces `identity_gateway_api_keys` as the canonical application access key table. It stores:

- `hashed_key`
- `raw_key`
- `tenant_id`
- `project_id`
- `environment`
- `label`
- `notes`
- `created_at_ms`
- `last_used_at_ms`
- `expires_at_ms`
- `active`

`raw_key` is added to satisfy the new persistence requirement. Runtime authentication still uses `hashed_key` lookup from the presented plaintext key.

## Runtime Compatibility Strategy

- Storage implementations migrate legacy rows into canonical `ai_*` tables during startup.
- Legacy table names remain queryable through compatibility views so direct SQL-based tests and older tooling do not fail during the transition.
- Existing runtime/store interfaces such as `list_models()` continue returning provider-scoped `ModelCatalogEntry` objects by joining `ai_model` with `ai_model_price`.
- Admin API gains new canonical endpoints for channel models and model pricing while legacy model CRUD can continue to function as an adapter over the new tables.

## Bootstrap Data Domains

The bootstrap system is profile-driven and update-pack-driven. Repository seed data stays grouped under `/data/<domain>/*.json`, with ordered profile manifests in `/data/profiles/*.json` and additive update manifests in `/data/updates/*.json`.

The bootstrap framework now also treats the commercial account kernel as first-class repository data. The key domains are:

- `accounts`
- `account-benefit-lots`
- `account-holds`
  carries `holds` plus `allocations`
- `account-ledger`
  carries `entries` plus `allocations`
- `request-metering`
  carries `facts` plus `metrics`
- `request-settlements`
- `account-reconciliation`

Runtime governance uses three dedicated domains so plugin/runtime coordination stays high-cohesion and low-coupling:

- `service-runtime-nodes`
  Stores `ServiceRuntimeNodeRecord` arrays.
  Each node is idempotently keyed by `node_id`.
- `extension-runtime-rollouts`
  Stores rollout bundles with `rollouts` and `participants`.
  Rollouts reference extension ids and/or extension instances.
  Participants reference runtime nodes and must keep `service_kind` aligned with the target node.
- `standalone-config-rollouts`
  Stores rollout bundles with `rollouts` and `participants`.
  Rollouts may target a specific `requested_service_kind` or remain global.
  Participants reference runtime nodes and must satisfy the rollout's requested service scope when one is declared.

This keeps catalog/bootstrap semantics intact:

- `channel` remains the canonical model inventor/vendor.
- `provider` remains the official, proxy, or local execution endpoint.
- `route config` still selects providers instead of channels.
- `model-price` remains the provider-specific pricing contract for a canonical channel model.
- `pricing-plan` and `pricing-rate` remain the internal commercial billing contract.
- request metering must reference a valid account, provider, channel, model, and optional pricing plans.
- proxy providers remain explicit model subsets through `provider-model`; they do not imply full channel coverage.
- Runtime governance records are independent from catalog records except for explicit references to extension instances and extension ids.

Update packs remain last-wins and idempotent, so bootstrap can be rerun during startup, deployment, or upgrade without producing duplicate governance records or cross-domain drift.

## Admin API Changes

### New canonical routes

- `GET/POST /admin/channel-models`
- `DELETE /admin/channel-models/{channel_id}/models/{model_id}`
- `GET/POST /admin/model-prices`
- `DELETE /admin/model-prices/{channel_id}/models/{model_id}/providers/{proxy_provider_id}`

### Compatibility behavior

- Existing `/admin/models` create/delete behavior remains as an adapter that materializes `ai_model` and `ai_model_price`.
- `/admin/channels`, `/admin/providers`, `/admin/credentials`, `/admin/api-keys` remain available but backed by canonical `ai_*` tables.
- Provider-model coverage remains the control point for proxy-provider subset management, so admin workflows must keep provider coverage editing separate from canonical channel-model creation.

## Admin UI Changes

The catalog page becomes a channel-first workspace:

- Channel table with add/edit/delete and per-channel “Manage models” action.
- Channel model modal with model CRUD.
- Nested model pricing modal with provider-aware CRUD for official/proxy/local pricing and tier metadata.
- Dedicated proxy provider table with CRUD.
- Proxy provider workflows must let operators choose which canonical models a proxy provider actually exposes instead of assuming full channel coverage.
- Router credential inventory preserved with CRUD.
- App API key registry remains in the tenant/traffic surface, but schema changes are reflected in the API type layer.

## Verification Plan

- Storage migration tests assert canonical `ai_*` tables exist and legacy compatibility views remain readable.
- Storage catalog tests assert channel models and model prices round-trip and provider variants are synthesized correctly.
- Admin route tests cover:
  - canonical channel model CRUD
  - canonical model price CRUD
  - default channel seed rows
  - `raw_key` persistence in `ai_app_api_keys`
- Admin UI tests assert:
  - channel table exposes model management
  - model modal exposes provider pricing CRUD
  - proxy provider CRUD remains available
