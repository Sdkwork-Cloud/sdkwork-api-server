# Software Architecture

This page explains how SDKWork API Server is assembled as a product and as a codebase.

## System View

| Surface | Primary responsibility | Primary paths |
|---|---|---|
| gateway | OpenAI-compatible data plane and provider dispatch | `services/gateway-service`, `crates/sdkwork-api-interface-http` |
| admin | operator control plane | `services/admin-api-service`, `crates/sdkwork-api-interface-admin` |
| portal | self-service user boundary | `services/portal-api-service`, `crates/sdkwork-api-interface-portal` |
| web host | Pingora public web delivery and API proxy | `services/router-web-service`, `crates/sdkwork-api-runtime-host` |
| product host | integrated web + API host for desktop and server mode | `services/router-product-service`, `crates/sdkwork-api-product-runtime` |
| admin app | browser and Tauri operator experience | `apps/sdkwork-router-admin/` |
| portal app | browser self-service experience | `apps/sdkwork-router-portal/` |
| docs | documentation product | `docs/` |

## Request Flow

For a typical stateful gateway request, the flow is:

1. a client calls a `/v1/*` route on `gateway-service`
2. `sdkwork-api-interface-http` authenticates the gateway key and maps the HTTP route
3. application crates resolve models, providers, credentials, routing policy, quota, and runtime state
4. a provider runtime executes through builtin, connector, or native-dynamic dispatch
5. usage and billing records are persisted through the admin-store contract
6. the OpenAI-compatible response is returned to the caller

The admin and portal services follow the same broad structure, but they terminate in native control-plane workflows instead of provider dispatch.

## Workspace Layering

| Layer | Responsibility | Examples |
|---|---|---|
| interface | HTTP routers, auth boundaries, response shaping | `sdkwork-api-interface-http`, `sdkwork-api-interface-admin`, `sdkwork-api-interface-portal` |
| app | use-case orchestration and service workflows | `sdkwork-api-app-gateway`, `sdkwork-api-app-routing`, `sdkwork-api-app-billing`, `sdkwork-api-app-extension` |
| domain | policy rules and durable concepts | `sdkwork-api-domain-routing`, `sdkwork-api-domain-billing`, `sdkwork-api-domain-usage` |
| storage | persistence contracts and concrete backends | `sdkwork-api-storage-core`, `sdkwork-api-storage-sqlite`, `sdkwork-api-storage-postgres` |
| provider | upstream adapter implementations | `sdkwork-api-provider-openai`, `sdkwork-api-provider-openrouter`, `sdkwork-api-provider-ollama` |
| runtime | extension loading, embedded hosting, supervision | `sdkwork-api-runtime-host`, `sdkwork-api-extension-host`, `sdkwork-api-app-runtime` |
| contracts | OpenAI and gateway API shapes | `sdkwork-api-contract-openai`, `sdkwork-api-contract-gateway` |

## Runtime Architecture

SDKWork supports both standalone and embedded runtime shapes.

- standalone services bind independent HTTP listeners for gateway, admin, and portal
- `router-web-service` fronts the admin and portal static sites and proxies `/api/admin/*`, `/api/portal/*`, and `/api/v1/*`
- `sdkwork-api-product-runtime` composes the standalone listeners plus the shared runtime host into one product-level runtime
- `router-product-service` is the deployable server entrypoint for the integrated product runtime
- the admin and portal Tauri hosts both embed the shared product runtime so desktop mode still exposes browser-facing admin and portal URLs
- product runtime role slicing supports cluster deployment where `web`, `gateway`, `admin`, and `portal` can live on different nodes
- extension runtimes can be builtin, connector-based, or native-dynamic
- runtime configuration, listener rebinding, store swaps, and secret-manager rotation are designed for hot reload instead of full process restarts

For the runtime deep dive, see [Runtime Modes Deep Dive](/architecture/runtime-modes).

## Configuration and Secret Boundaries

Configuration is loaded through `sdkwork-api-config` and then applied to the process and runtime handles.

Important boundaries:

- service bind addresses
- database backend and connection string
- admin and portal JWT signing secrets
- provider credential master keys
- extension discovery paths and trust policy

Secret material is intentionally separated from route logic:

- services resolve secret-manager strategy from config
- credentials are stored through encrypted backends
- historical credential readability is preserved through persisted locator and key-lineage metadata

## Persistence Model

The shared admin store is the system-of-record for:

- tenants and projects
- gateway API keys
- channels, providers, credentials, and models
- routing policies and decision logs
- usage records, billing ledger entries, and quota policies
- extension installation and rollout state

Backends currently documented in-repo:

- SQLite
- PostgreSQL

## Frontend Architecture

The frontend control-plane product is intentionally split into two independent applications:

- `apps/sdkwork-router-admin/` owns the super-admin UX, admin API client, and Tauri host
- `apps/sdkwork-router-portal/` owns the self-service workspace UX, portal API client, and server-mode product entrypoint
- `crates/sdkwork-api-runtime-host` owns the shared Pingora delivery boundary used by server mode and desktop mode
- `crates/sdkwork-api-product-runtime` owns the shared bootstrap that starts local API listeners and then publishes the integrated web surface

This keeps operator and self-service concerns isolated while preserving a single public delivery contract.

## Operational Architecture

All three standalone services expose:

- health endpoints
- Prometheus-style metrics
- structured request tracing

The integrated product host adds:

- one public bind for `/admin/*`, `/portal/*`, and `/api/*`
- environment-driven role slicing for cluster topology
- explicit upstream wiring when `web` is deployed without a colocated API role

The admin control plane additionally owns:

- extension runtime reloads and rollouts
- standalone config rollouts
- routing simulations
- health snapshot inspection
- usage and billing visibility

## Related Docs

- module ownership:
  - [Functional Modules](/architecture/functional-modules)
- endpoint inventory:
  - [API Reference Overview](/api-reference/overview)
- config and observability:
  - [Configuration](/operations/configuration)
  - [Health and Metrics](/operations/health-and-metrics)
