# AI Router Schema Standardization Design

## Goal

Standardize the sdkwork-api-router control-plane schema around canonical `ai_*` table names, preserve backward compatibility for legacy SQL callers, add first-class channel/model/provider pricing management, and extend the standalone admin application so operators can manage channels, models, model pricing, proxy providers, router credentials, and app API keys from one coherent catalog workflow.

## Current State

- Canonical data is currently spread across mixed table families such as `identity_*`, `tenant_*`, `catalog_*`, `routing_*`, `billing_*`, and `extension_*`.
- Runtime-facing model records are provider-centric (`external_name + provider_id`) while the product requirement is channel-centric (`channel -> models -> provider pricing`).
- Gateway app API keys only persist hashed values in storage, while the product requirement now asks for persisted `raw_key` plus `hashed_key`.
- Router upstream credentials are stored in `credential_records`; the product requirement wants the router config secret registry to live in `ai_router_credential_records`.
- The admin catalog page exposes channel/provider/credential/model CRUD, but it does not support channel-scoped model management or provider-aware pricing CRUD.

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
- `ai_model_price`
- `ai_routing_policies`
- `ai_routing_policy_provider`
- `ai_project_routing_preferences`
- `ai_routing_decision_logs`
- `ai_provider_health_records`
- `ai_usage_records`
- `ai_billing_ledger_entries`
- `ai_billing_quota_policies`
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

## Admin API Changes

### New canonical routes

- `GET/POST /admin/channel-models`
- `DELETE /admin/channel-models/{channel_id}/models/{model_id}`
- `GET/POST /admin/model-prices`
- `DELETE /admin/model-prices/{channel_id}/models/{model_id}/providers/{proxy_provider_id}`

### Compatibility behavior

- Existing `/admin/models` create/delete behavior remains as an adapter that materializes `ai_model` and `ai_model_price`.
- `/admin/channels`, `/admin/providers`, `/admin/credentials`, `/admin/api-keys` remain available but backed by canonical `ai_*` tables.

## Admin UI Changes

The catalog page becomes a channel-first workspace:

- Channel table with add/edit/delete and per-channel “Manage models” action.
- Channel model modal with model CRUD.
- Nested model pricing modal with provider-aware CRUD for input/output/cache pricing.
- Dedicated proxy provider table with CRUD.
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
