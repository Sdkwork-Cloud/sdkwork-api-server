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

Password rotation:

```bash
curl -X POST http://127.0.0.1:8081/admin/auth/change-password \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <jwt>" \
  -d '{
    "current_password":"ChangeMe123!",
    "new_password":"AdminPassword456!"
  }'
```

## Route Families

## OpenAPI Inventory

- OpenAPI JSON: `GET /admin/openapi.json`
- API inventory UI: `GET /admin/docs`

The OpenAPI document is generated from the current `axum` router so the listed paths track the live admin service surface.

| Family | Routes | Purpose |
|---|---|---|
| health and metrics | `GET /admin/health`, `GET /metrics` | liveness and Prometheus-style metrics |
| auth | `POST /auth/login`, `GET /auth/me`, `POST /auth/change-password` | operator authentication and password rotation |
| tenancy | `GET/POST /tenants`, `GET/POST /projects` | tenant and project lifecycle |
| gateway access | `GET/POST /api-keys` | gateway API key issuance and listing |
| provider catalog | `GET/POST /channels`, `GET/POST /providers`, `GET/POST /credentials`, `GET/POST /models` | upstream ecosystem definition |
| extensions | `GET/POST /extensions/installations`, `GET /extensions/packages`, `GET/POST /extensions/instances`, `GET /extensions/runtime-statuses`, `POST /extensions/runtime-reloads` | extension runtime management |
| extension rollouts | `GET/POST /extensions/runtime-rollouts`, `GET /extensions/runtime-rollouts/{rollout_id}` | coordinated extension rollout control |
| runtime config rollouts | `GET/POST /runtime-config/rollouts`, `GET /runtime-config/rollouts/{rollout_id}` | coordinated config reload control |
| usage and billing | `GET /usage/records`, `GET /usage/summary`, `GET /billing/ledger`, `GET /billing/summary`, `GET/POST /billing/quota-policies` | operator observability and enforcement |
| routing | `GET/POST /routing/policies`, `GET /routing/health-snapshots`, `GET /routing/decision-logs`, `POST /routing/simulations` | dispatch policy and diagnostics |

## What The Admin API Owns

The admin API is the system-of-record surface for:

- providers and credentials
- model catalog
- routing policy
- runtime rollout state
- usage and billing summaries
- quota controls

If you need to operate the gateway, this is the API that changes the underlying behavior.

## Browser App

The operator UI is a dedicated browser app:

- `http://127.0.0.1:5173/admin/`

## Related Docs

- service ownership:
  - [API Reference Overview](/api-reference/overview)
- self-service end-user boundary:
  - [Portal API](/api-reference/portal-api)
- architecture context:
  - [Software Architecture](/architecture/software-architecture)
