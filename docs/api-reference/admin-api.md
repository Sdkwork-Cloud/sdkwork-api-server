# Admin API

The admin service exposes the operator control plane under `/admin/*`.

## Base URL and Auth

- default local base URL: `http://127.0.0.1:8081/admin`
- auth flow:
  - `POST /admin/auth/login`
  - `GET /admin/auth/me`
  - `POST /admin/auth/change-password`

Example login:

```bash
curl -X POST http://127.0.0.1:8081/admin/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"admin@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

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
- `ai_billing_events`: canonical Billing 2.0 event ledger for multimodal and routing-aware accounting

Unified app API keys are persisted in `ai_app_api_keys` with:

- `hashed_key`: lookup key used for runtime authentication
- `raw_key`: persisted original plaintext key when retained by policy
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
| gateway access | `GET/POST /api-keys`, `PUT /api-keys/{hashed_key}`, `POST /api-keys/{hashed_key}/status`, `DELETE /api-keys/{hashed_key}` | gateway API key issuance, metadata updates, raw key visibility, and status control |
| gateway groups | `GET/POST /api-key-groups`, `PATCH /api-key-groups/{group_id}`, `POST /api-key-groups/{group_id}/status`, `DELETE /api-key-groups/{group_id}` | API key group lifecycle, policy attachment prep, and activation control |
| provider catalog | `GET/POST /channels`, `DELETE /channels/{channel_id}`, `GET/POST /providers`, `DELETE /providers/{provider_id}`, `GET/POST /credentials`, `DELETE /credentials/{tenant_id}/providers/{provider_id}/keys/{key_reference}` | channel, provider, and credential management |
| channel models | `GET/POST /channel-models`, `DELETE /channel-models/{channel_id}/models/{model_id}` | channel-to-model mapping management |
| model pricing | `GET/POST /model-prices`, `DELETE /model-prices/{channel_id}/models/{model_id}/providers/{proxy_provider_id}` | per-channel-model, per-provider pricing control |
| marketing workbench | `GET/POST /marketing/coupon-templates`, `POST /marketing/coupon-templates/{coupon_template_id}/status`, `GET/POST /marketing/campaigns`, `POST /marketing/campaigns/{marketing_campaign_id}/status`, `GET/POST /marketing/budgets`, `POST /marketing/budgets/{campaign_budget_id}/status`, `GET/POST /marketing/codes`, `POST /marketing/codes/{coupon_code_id}/status`, `GET /marketing/reservations`, `GET /marketing/redemptions`, `GET /marketing/rollbacks` | canonical coupon governance owned only by `/admin/marketing/*` |
| compatibility model routes | `GET/POST /models`, `DELETE /models/{external_name}/providers/{provider_id}` | legacy provider-scoped model compatibility routes backed by canonical catalog tables |
| extensions | `GET/POST /extensions/installations`, `GET /extensions/packages`, `GET/POST /extensions/instances`, `GET /extensions/runtime-statuses`, `POST /extensions/runtime-reloads` | extension runtime management |
| extension rollouts | `GET/POST /extensions/runtime-rollouts`, `GET /extensions/runtime-rollouts/{rollout_id}` | coordinated extension rollout control |
| runtime config rollouts | `GET/POST /runtime-config/rollouts`, `GET /runtime-config/rollouts/{rollout_id}` | coordinated config reload control |
| usage and billing | `GET /usage/records`, `GET /usage/summary`, `GET /billing/events`, `GET /billing/events/summary`, `GET /billing/ledger`, `GET /billing/summary`, `GET /billing/accounts`, `GET /billing/accounts/{account_id}/balance`, `GET /billing/accounts/{account_id}/benefit-lots`, `GET /billing/account-holds`, `GET /billing/request-settlements`, `POST /billing/pricing-lifecycle/synchronize`, `GET/POST /billing/pricing-plans`, `PUT /billing/pricing-plans/{pricing_plan_id}`, `POST /billing/pricing-plans/{pricing_plan_id}/clone`, `POST /billing/pricing-plans/{pricing_plan_id}/schedule`, `POST /billing/pricing-plans/{pricing_plan_id}/publish`, `POST /billing/pricing-plans/{pricing_plan_id}/retire`, `GET/POST /billing/pricing-rates`, `PUT /billing/pricing-rates/{pricing_rate_id}`, `GET/POST /billing/quota-policies` | operator observability, legacy quota posture, and canonical commercial account investigation |
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

## Marketing Workbench Notes

- `POST /admin/marketing/coupon-templates/{coupon_template_id}/status` updates the lifecycle state of the canonical coupon template without forcing operators to rewrite the rest of the record.
- `POST /admin/marketing/campaigns/{marketing_campaign_id}/status` controls campaign activation and pause workflows for the attached template.
- `POST /admin/marketing/budgets/{campaign_budget_id}/status` closes or re-activates a canonical budget while preserving consumed and reserved counters for auditability.
- `POST /admin/marketing/codes/{coupon_code_id}/status` disables or re-enables a canonical coupon code without deleting redemption history.
- `/admin/coupons` has been removed. Coupon governance and coupon-facing lifecycle control now exist only under canonical `/admin/marketing/*` resources.

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

## Canonical Commercial Account Notes

- `GET /admin/billing/accounts` lists canonical payable accounts together with summarized available, held, consumed, and granted balance posture
- `GET /admin/billing/accounts/{account_id}/balance` returns the full computed balance snapshot, including active spendable lots
- `GET /admin/billing/accounts/{account_id}/benefit-lots` exposes all canonical lots for operator investigation, including expired or exhausted lots that do not appear in the active balance snapshot
- `GET /admin/billing/account-holds` exposes canonical request admission holds
- `GET /admin/billing/request-settlements` exposes canonical request settlement outcomes
- `GET /admin/billing/pricing-plans` and `GET /admin/billing/pricing-rates` expose the pricing posture that commercial account settlement will consume as the router moves further off legacy quota mode
- admin pricing reads now synchronize due `planned` versions before returning data, so once a staged plan reaches `effective_from_ms` the latest due version becomes `active` and replaced active siblings are archived automatically
- `POST /admin/billing/pricing-lifecycle/synchronize` exposes the same lifecycle convergence as an explicit control-plane operation and returns due-group, activated, archived, skipped, and timestamp counters for operator review or automation
- `POST /admin/billing/pricing-plans` creates canonical pricing-plan headers with plan code, version, currency, credit unit, lifecycle status, and effective window metadata via `effective_from_ms` and optional `effective_to_ms`
- `PUT /admin/billing/pricing-plans/{pricing_plan_id}` fully replaces an existing canonical pricing-plan header so operators can advance plan versions, lifecycle state, and effective windows without recreating identities in the UI
- `POST /admin/billing/pricing-plans/{pricing_plan_id}/clone` creates the next draft version of an existing pricing plan and clones all attached pricing-rate rows onto the new plan identity
- `POST /admin/billing/pricing-plans/{pricing_plan_id}/schedule` marks a future-dated pricing plan as `planned` and stages its attached pricing-rate rows with the same lifecycle status while leaving the currently active sibling version untouched
- `POST /admin/billing/pricing-plans/{pricing_plan_id}/publish` promotes a pricing plan to `active`, activates its attached pricing-rate rows, and archives previously active sibling versions for the same tenant, organization, and plan code; only currently effective plans can be published immediately
- `POST /admin/billing/pricing-plans/{pricing_plan_id}/retire` archives the target pricing plan and all attached pricing-rate rows without deleting the commercial pricing history
- `POST /admin/billing/pricing-rates` creates canonical pricing-rate rows with explicit `charge_unit`, `pricing_method`, display unit, rounding metadata, minimums, and optional capability or provider or model scope
- `PUT /admin/billing/pricing-rates/{pricing_rate_id}` fully replaces an existing canonical pricing-rate row so settlement-facing pricing semantics stay editable after first publication
- canonical pricing rates now support professional commercial units such as input tokens, requests, images, audio seconds, video minutes, and music tracks without overloading the catalog market-price model

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
