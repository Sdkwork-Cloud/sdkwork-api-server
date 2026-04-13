# Admin API

The admin service exposes the operator control plane under `/admin/*`.

## Base URL and Auth

- default local base URL: `http://127.0.0.1:8081/admin`
- auth flow:
  - `POST /admin/auth/login`
  - `GET /admin/auth/me`
  - `POST /admin/auth/change-password`

Example login for an explicitly enabled local development bootstrap:

```bash
curl -X POST http://127.0.0.1:8081/admin/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"admin@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

The built-in `admin@sdkwork.local / ChangeMe123!` bootstrap account is available only when startup enables `SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP=true`. In secure runtime mode, create operator accounts explicitly instead of relying on demo credentials.

Use the returned JWT as:

```bash
-H "Authorization: Bearer <jwt>"
```

Minimal verification:

```bash
curl http://127.0.0.1:8081/admin/auth/me \
  -H "Authorization: Bearer <jwt>"
```

## OpenAPI Inventory

- OpenAPI JSON: `GET /admin/openapi.json`
- API inventory UI: `GET /admin/docs`

The OpenAPI document is generated from the current `axum` router so the listed paths track the live admin service surface.

## Canonical Persistence

The router control plane now standardizes catalog and credential persistence around canonical `ai_*` tables with lowercase snake_case fields.

Key tables:

- `ai_channel`: canonical channel catalog, seeded with `openai`, `anthropic`, `gemini`, `openrouter`, and `ollama`
- `ai_proxy_provider`: canonical proxy provider registry
- `ai_proxy_provider_channel`: channel bindings for each proxy provider
- `ai_model`: channel-to-model mapping registry
- `ai_model_price`: channel-model to proxy-provider pricing registry
- `ai_router_credential_records`: encrypted router credential storage
- `ai_app_api_key_groups`: application API key group registry
- `ai_app_api_keys`: application access key registry
- `ai_admin_audit_events`: durable operator audit ledger for high-risk control-plane writes
- `ai_billing_events`: canonical Billing 2.0 event ledger for multimodal and routing-aware accounting

Unified app API keys are persisted in `ai_app_api_keys` with:

- `hashed_key`: lookup key used for runtime authentication
- `key_prefix`: non-secret prefix retained for operator recognition and audit trails
- `tenant_id`
- `project_id`
- `environment`
- `api_key_group_id`
- `label`
- `notes`
- `created_at_ms`
- `last_used_at_ms`
- `expires_at_ms`
- `active`

Plaintext API keys are returned exactly once in the create response payload and are never serialized again from admin list or detail reads.

API key groups are persisted in `ai_app_api_key_groups` with:

- `group_id`
- `tenant_id`
- `project_id`
- `environment`
- `name`
- `slug`
- `description`
- `color`
- `default_capability_scope`
- `default_routing_profile_id`
- `active`
- `created_at_ms`
- `updated_at_ms`

Router upstream credentials are persisted in `ai_router_credential_records` with:

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

`secret_ciphertext` is the encrypted router config secret payload. Admin responses never return the submitted cleartext secret value.

Fresh databases now create only `ai_*` physical tables. Legacy names such as `identity_gateway_api_keys`, `credential_records`, `catalog_channels`, and `identity_users` are migrated into the canonical tables during startup and then re-exposed as compatibility views so existing SQL tooling keeps working.

## Route Families

| Family | Routes | Purpose |
|---|---|---|
| health and metrics | `GET /admin/health`, `GET /metrics` | liveness and Prometheus-style metrics |
| auth | `POST /auth/login`, `GET /auth/me`, `POST /auth/change-password` | operator authentication and password rotation |
| tenancy | `GET/POST /tenants`, `DELETE /tenants/{tenant_id}`, `GET/POST /projects`, `DELETE /projects/{project_id}` | tenant and project lifecycle |
| gateway access | `GET/POST /api-keys`, `PUT /api-keys/{hashed_key}`, `POST /api-keys/{hashed_key}/status`, `DELETE /api-keys/{hashed_key}` | gateway API key issuance, metadata updates, one-time plaintext reveal at create, and status control |
| gateway groups | `GET/POST /api-key-groups`, `PATCH /api-key-groups/{group_id}`, `POST /api-key-groups/{group_id}/status`, `DELETE /api-key-groups/{group_id}` | API key group lifecycle, policy attachment prep, and activation control |
| provider catalog | `GET/POST /channels`, `DELETE /channels/{channel_id}`, `GET/POST /providers`, `DELETE /providers/{provider_id}`, `GET/POST /credentials`, `DELETE /credentials/{tenant_id}/providers/{provider_id}/keys/{key_reference}` | channel, provider, and credential management |
| channel models | `GET/POST /channel-models`, `DELETE /channel-models/{channel_id}/models/{model_id}` | channel-to-model mapping management |
| model pricing | `GET/POST /model-prices`, `DELETE /model-prices/{channel_id}/models/{model_id}/providers/{proxy_provider_id}` | per-channel-model, per-provider pricing control |
| compatibility model routes | `GET/POST /models`, `DELETE /models/{external_name}/providers/{provider_id}` | legacy provider-scoped model compatibility routes backed by canonical catalog tables |
| audit and governance | `GET /audit/events` | durable operator audit history for identity, secret, and finance-sensitive mutations |
| extensions | `GET/POST /extensions/installations`, `GET /extensions/packages`, `GET/POST /extensions/instances`, `GET /extensions/runtime-statuses`, `POST /extensions/runtime-reloads` | extension runtime management |
| extension rollouts | `GET/POST /extensions/runtime-rollouts`, `GET /extensions/runtime-rollouts/{rollout_id}` | coordinated extension rollout control |
| runtime config rollouts | `GET/POST /runtime-config/rollouts`, `GET /runtime-config/rollouts/{rollout_id}` | coordinated config reload control |
| usage and billing | `GET /usage/records`, `GET /usage/summary`, `GET /billing/events`, `GET /billing/events/summary`, `GET /billing/ledger`, `GET /billing/summary`, `GET/POST /billing/quota-policies` | operator observability, billing-event inspection, effective balance visibility, and quota enforcement |
| routing | `GET/POST /routing/policies`, `GET/POST /routing/profiles`, `GET /routing/snapshots`, `GET /routing/health-snapshots`, `GET /routing/decision-logs`, `POST /routing/simulations` | dispatch policy, reusable routing profiles, compiled route state, and diagnostics |

## What The Admin API Owns

The admin API is the system-of-record surface for:

- channels, providers, credentials, and model pricing
- app API keys, API key groups, and router credential posture
- routing policy
- runtime rollout state
- usage and billing summaries
- quota controls

If you need to operate the gateway, this is the API that changes the underlying behavior.

## Role And Governance Notes

- `super_admin` owns identity and secret administration:
  - operator and portal user lifecycle
  - upstream credentials
  - gateway API keys
  - API key groups
- `finance_operator` owns finance-sensitive commercial writes:
  - model pricing
  - billing and marketing write routes
- `platform_operator` owns catalog, routing, runtime, and other platform-control writes, but cannot mutate model prices or secret-bearing routes
- `read_only_operator` can inspect platform and pricing read models plus the audit ledger, but cannot write control-plane state
- the current codebase does not yet expose admin-side refund, dispute, or payment-adjustment write routes; when those are added they should reuse the same durable audit substrate and finance-bound approval posture instead of creating a parallel governance path

## Audit Event Notes

- `GET /admin/audit/events` reads durable control-plane audit history from `ai_admin_audit_events`
- every audit event includes:
  - `actor_user_id`
  - `actor_email`
  - `actor_role`
  - `action`
  - `resource_type`
  - `resource_id`
  - `approval_scope`
  - optional `target_tenant_id`
  - optional `target_project_id`
  - optional `target_provider_id`
  - `created_at_ms`
- approval scopes currently emitted by the admin service include:
  - `identity_control`
  - `secret_control`
  - `finance_control`
- audited high-risk mutations now include:
  - admin user create, update, status change, password reset, and delete
  - portal user create, update, status change, password reset, and delete
  - gateway API key create, update, status change, and delete
  - API key group create, update, status change, and delete
  - upstream credential create and delete
  - model price create and delete

## API Key Group Notes

- `POST /admin/api-key-groups` creates a first-class API key group under a specific `tenant_id + project_id + environment`
- `slug` is optional on create or update; when omitted it is derived from `name`
- `default_routing_profile_id` is optional on create or update; when present it must resolve to an active routing profile in the same `tenant_id + project_id`
- `default_accounting_mode` is optional on create or update; when present it is normalized to one of:
  - `platform_credit`
  - `byok`
  - `passthrough`
- gateway billing-event inference now reads `default_accounting_mode` from the bound API key group when present and otherwise falls back to `platform_credit`
- `POST /admin/api-keys` and `PUT /admin/api-keys/{hashed_key}` now accept optional `api_key_group_id`
- group binding is rejected when the referenced group belongs to another tenant or project, has another environment, or is inactive

## Routing Profile Notes

- `POST /admin/routing/profiles` creates a reusable workspace-scoped routing profile bundle
- routing profiles reuse the same provider-order, strategy, cost, latency, health, and preferred-region semantics already used by project routing preferences
- `GET /admin/routing/snapshots` lists the current compiled route state derived from canonical policy records, project defaults, and optional group-bound routing profiles
- `POST /admin/routing/simulations` now accepts optional `tenant_id`, `project_id`, and `api_key_group_id` to preview group-bound routing behavior
- routing simulation responses and decision logs now surface `applied_routing_profile_id` when a group-bound profile participates in the final routing decision
- routing simulation responses now also surface:
  - `compiled_routing_snapshot_id`
  - `selected_candidate`
  - `rejected_candidates`
  - `fallback_reason`

## Billing Event Notes

- `GET /admin/billing/summary` now returns `remaining_units` as the effective project balance exposed to operators
- when canonical recharge accounts exist for a project, `GET /admin/billing/summary` sets `balance_source` to `canonical_account` and also exposes:
  - `quota_remaining_units`
  - `canonical_account_id`
  - `canonical_available_balance`
  - `canonical_held_balance`
  - `canonical_grant_balance`
  - `canonical_consumed_balance`
- when no canonical recharge account exists yet, `balance_source` remains `quota_policy` and `remaining_units` continues to reflect the quota layer
- `GET /admin/billing/events` exposes the canonical Billing 2.0 event ledger
- `GET /admin/billing/events/summary` aggregates billing events by:
  - project
  - API key group
  - capability
  - accounting mode
- billing events retain:
  - `api_key_group_id`
  - `route_key`
  - `usage_model`
  - `provider_id`
  - `channel_id`
  - `reference_id`
  - token, cache, and media dimensions
  - `upstream_cost`
  - `customer_charge`
  - `applied_routing_profile_id`
  - `compiled_routing_snapshot_id`
  - `fallback_reason`
- gateway metering now populates `image_count` for image generation flows and `music_seconds` for music creation flows; audio and video duration metrics remain available in the ledger schema for richer upstream request contracts
- gateway metering now dual-writes:
  - legacy usage records
  - legacy billing ledger entries
  - canonical billing events

## Browser App

The operator UI is a dedicated browser app:

- `http://127.0.0.1:5173/admin/`

The catalog module now supports:

- channel CRUD from a tabular registry
- dynamic model entry when creating or editing a channel
- per-channel "Manage models" workflow
- per-model "Manage pricing" workflow
- proxy provider CRUD
- router credential CRUD

## Related Docs

- service ownership:
  - [API Reference Overview](/api-reference/overview)
- self-service end-user boundary:
  - [Portal API](/api-reference/portal-api)
- architecture context:
  - [Software Architecture](/architecture/software-architecture)
