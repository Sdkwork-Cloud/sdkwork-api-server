# API Reference Overview

This documentation set is organized around three public HTTP surfaces:

- the OpenAI-compatible gateway
- the native admin control plane
- the native public portal

## Base Paths and Ownership

| Surface | Default local base URL | Primary auth model | Use it for |
|---|---|---|---|
| gateway | `http://127.0.0.1:8080/v1` | gateway API key | application traffic and model execution |
| admin | `http://127.0.0.1:8081/admin` | admin JWT | operators, control plane, billing, routing, runtime control |
| portal | `http://127.0.0.1:8082/portal` | portal JWT | end users, self-service auth, workspace, API keys |

## Authentication Boundaries

| Surface | Header | Token issuer |
|---|---|---|
| gateway | `Authorization: Bearer skw_live_...` | gateway API key store |
| admin | `Authorization: Bearer <jwt>` | admin auth login flow |
| portal | `Authorization: Bearer <jwt>` | portal auth login flow |

Admin and portal tokens are intentionally separate. A portal session is not an admin session.

## API Conventions

- gateway routes are designed to be OpenAI-compatible where documented
- admin and portal routes are SDKWork-owned native APIs
- standalone services expose `/metrics` and health endpoints outside the primary API namespaces
- request tracing propagates `x-request-id`
- gateway routing can take an optional `x-sdkwork-region` hint for geo-affinity-aware routing decisions

## Compatibility Labels

SDKWork uses five execution-truth labels:

| Label | Meaning |
|---|---|
| `native` | implemented directly inside SDKWork |
| `relay` | forwarded to an upstream compatible provider |
| `translated` | accepted locally but mapped to a different upstream primitive |
| `emulated` | returned locally in a compatible shape |
| `unsupported` | not available in the current runtime |

See [API Compatibility](/reference/api-compatibility) and the [Full Compatibility Matrix](/api/compatibility-matrix) for details.

## First Authenticated Calls

Admin login:

```bash
curl -X POST http://127.0.0.1:8081/admin/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"admin@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

Portal default login:

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

Portal registration:

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@example.com",
    "password":"PortalPass123!",
    "display_name":"Portal User"
  }'
```

Gateway first request:

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer skw_live_your_key_here"
```

For the full bootstrap path, see [Quickstart](/getting-started/quickstart).

## Browser Apps

The browser surface is intentionally split into two independent applications:

- admin app: `http://127.0.0.1:5173/admin/`
- portal app: `http://127.0.0.1:5174/`

## Service-Specific Reference Pages

- [Gateway API](/api-reference/gateway-api)
- [Admin API](/api-reference/admin-api)
- [Portal API](/api-reference/portal-api)

## OpenAI Reference Alignment

SDKWork intentionally mirrors the structure of OpenAI's platform documentation for the data plane. For upstream schema and resource semantics, the official OpenAI docs remain the primary reference:

- OpenAI API reference: <https://platform.openai.com/docs/api-reference>
- OpenAI docs overview: <https://platform.openai.com/docs/overview>
