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
- `ai_app_api_keys`: application access key registry

Unified app API keys are persisted in `ai_app_api_keys` with:

- `hashed_key`: lookup key used for runtime authentication
- `raw_key`: persisted original plaintext key when retained by policy
- `tenant_id`
- `project_id`
- `environment`
- `label`
- `notes`
- `created_at_ms`
- `last_used_at_ms`
- `expires_at_ms`
- `active`

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
| gateway access | `GET/POST /api-keys`, `POST /api-keys/{hashed_key}/status`, `DELETE /api-keys/{hashed_key}` | gateway API key issuance, raw key visibility, and status control |
| provider catalog | `GET/POST /channels`, `DELETE /channels/{channel_id}`, `GET/POST /providers`, `DELETE /providers/{provider_id}`, `GET/POST /credentials`, `DELETE /credentials/{tenant_id}/providers/{provider_id}/keys/{key_reference}` | channel, provider, and credential management |
| channel models | `GET/POST /channel-models`, `DELETE /channel-models/{channel_id}/models/{model_id}` | channel-to-model mapping management |
| model pricing | `GET/POST /model-prices`, `DELETE /model-prices/{channel_id}/models/{model_id}/providers/{proxy_provider_id}` | per-channel-model, per-provider pricing control |
| compatibility model routes | `GET/POST /models`, `DELETE /models/{external_name}/providers/{provider_id}` | legacy provider-scoped model compatibility routes backed by canonical catalog tables |
| extensions | `GET/POST /extensions/installations`, `GET /extensions/packages`, `GET/POST /extensions/instances`, `GET /extensions/runtime-statuses`, `POST /extensions/runtime-reloads` | extension runtime management |
| extension rollouts | `GET/POST /extensions/runtime-rollouts`, `GET /extensions/runtime-rollouts/{rollout_id}` | coordinated extension rollout control |
| runtime config rollouts | `GET/POST /runtime-config/rollouts`, `GET /runtime-config/rollouts/{rollout_id}` | coordinated config reload control |
| usage and billing | `GET /usage/records`, `GET /usage/summary`, `GET /billing/ledger`, `GET /billing/summary`, `GET/POST /billing/quota-policies` | operator observability and enforcement |
| routing | `GET/POST /routing/policies`, `GET /routing/health-snapshots`, `GET /routing/decision-logs`, `POST /routing/simulations` | dispatch policy and diagnostics |

## What The Admin API Owns

The admin API is the system-of-record surface for:

- channels, providers, credentials, and model pricing
- app API keys and router credential posture
- routing policy
- runtime rollout state
- usage and billing summaries
- quota controls

If you need to operate the gateway, this is the API that changes the underlying behavior.

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
