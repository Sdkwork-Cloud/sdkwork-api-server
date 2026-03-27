# Repository Layout

This page explains how the workspace is organized so contributors can quickly find the right layer before making changes.

## Top-Level Structure

```text
.
|-- crates/
|-- services/
|-- apps/
|-- docs/
|-- scripts/
|-- Cargo.toml
|-- README.md
`-- README.zh-CN.md
```

## Runtime Surfaces

| Path | Responsibility |
|---|---|
| `services/gateway-service` | standalone `/v1/*` gateway binary |
| `services/admin-api-service` | standalone `/admin/*` control-plane binary |
| `services/portal-api-service` | standalone `/portal/*` self-service binary |
| `services/router-web-service` | Pingora public web host for admin and portal static delivery |
| `services/router-product-service` | integrated product host for server-mode `/admin/*`, `/portal/*`, and `/api/*` |
| `apps/sdkwork-router-admin/` | standalone admin browser app plus admin-owned Tauri host |
| `apps/sdkwork-router-portal/` | standalone browser portal app plus portal-owned Tauri host and product entrypoint |
| `docs/` | VitePress documentation site |

## Backend Layers

| Layer | Paths | Responsibility |
|---|---|---|
| interface | `crates/sdkwork-api-interface-*` | HTTP routers, request mapping, auth boundaries |
| app | `crates/sdkwork-api-app-*` | orchestration, workflow logic, service-level decisions |
| domain | `crates/sdkwork-api-domain-*` | domain models, policy rules, invariants |
| storage | `crates/sdkwork-api-storage-*` | persistence contracts and concrete backends |
| contracts | `crates/sdkwork-api-contract-*` | API shapes, compatibility contracts, shared request or response types |
| provider | `crates/sdkwork-api-provider-*` | upstream adapters and provider-specific execution |
| runtime | `crates/sdkwork-api-app-runtime`, `crates/sdkwork-api-runtime-host`, `crates/sdkwork-api-extension-*` | runtime loading, supervision, extension ABI, embedded hosting |
| cross-cutting | `crates/sdkwork-api-config`, `crates/sdkwork-api-observability`, `crates/sdkwork-api-kernel` | config, telemetry, runtime glue |

## Standalone Services

- `services/gateway-service`
- `services/admin-api-service`
- `services/portal-api-service`
- `services/router-web-service`
- `services/router-product-service`

## Frontend Layers

| Path | Responsibility |
|---|---|
| `apps/sdkwork-router-admin/src/` | standalone admin root shell and theme |
| `apps/sdkwork-router-admin/packages/` | admin foundation and business modules |
| `apps/sdkwork-router-admin/src-tauri/` | admin-owned Tauri host and desktop packaging integration |
| `apps/sdkwork-router-portal/src/` | standalone portal root shell and theme |
| `apps/sdkwork-router-portal/packages/` | portal foundation and business modules |

## Docs and Operational Assets

- `docs/`
  - VitePress docs site plus deep technical references
- `docs/plans/`
  - historical design and implementation records
- `scripts/dev/`
  - cross-platform startup helpers

## Common Navigation Rules

- changes to HTTP routes usually begin in `crates/sdkwork-api-interface-*`
- routing, billing, provider, or execution behavior usually continues in `crates/sdkwork-api-app-*`
- policy rules belong in `crates/sdkwork-api-domain-*`
- persistence or migration work belongs in `crates/sdkwork-api-storage-*`
- documentation and operator guidance belong in `docs/`
