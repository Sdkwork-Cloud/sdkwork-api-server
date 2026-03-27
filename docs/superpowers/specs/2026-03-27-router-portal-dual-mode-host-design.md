# Router Portal Dual-Mode Host Design

**Date:** 2026-03-27

**Goal**

Make `apps/sdkwork-router-portal` a real dual-mode product:
- desktop app via Tauri
- server-side product host via CLI/service process
- one shared Rust orchestration layer that can launch `gateway`, `admin`, `portal`, and `web`
- support single-node all-in-one deployment and cluster-friendly role slicing

**Current State**

- `sdkwork-router-portal` and `sdkwork-router-admin` each embed `sdkwork-api-runtime-host` directly from Tauri with hard-coded ports.
- API services already exist as separate processes:
  - `gateway-service`
  - `admin-api-service`
  - `portal-api-service`
  - `router-web-service`
- `sdkwork-api-app-runtime` already provides listener supervision, config reload, rollout heartbeats, and node coordination.
- `sdkwork-api-runtime-host` already provides the public web entrypoint that serves `/portal/*`, `/admin/*`, and proxies `/api/*`.

**Problems**

- The desktop entrypoints duplicate runtime boot logic.
- There is no shared “product host” that can start the full application stack in one process.
- There is no single service binary for `sdkwork-router-portal` server mode.
- Desktop mode hard-codes ports, which is fragile and not packaging-friendly.
- Tauri bundles do not currently include both admin and portal static assets for runtime web serving.

**Target Architecture**

Introduce a new shared crate: `crates/sdkwork-api-product-runtime`.

It owns:
- product roles: `web`, `gateway`, `admin`, `portal`
- runtime mode: `desktop` or `server`
- local listener planning
- optional external upstream overrides for cluster topologies
- unified bootstrap for:
  - shared store and secret manager handles
  - `gateway/admin/portal` routers
  - rollout supervision and runtime node heartbeats
  - public web host

**Desktop Mode**

- Use loopback-only listeners.
- Use ephemeral local ports for embedded services to avoid collisions.
- Start the public web host on `127.0.0.1:0` and expose the resolved base URL back to Tauri.
- Keep runtime node heartbeats and reload supervision by creating an override config loader with the resolved ephemeral binds.
- Resolve static assets from:
  1. bundled Tauri resources in packaged builds
  2. workspace `dist` directories as local fallback

**Server Mode**

- Run as a normal service binary.
- Default roles: `web,gateway,admin,portal`.
- Allow slicing roles with env configuration so one node can run only a subset.
- If `web` is enabled but a local API role is disabled, require an explicit upstream override for that API.
- Keep cluster capability by relying on the existing DB-backed service node heartbeats and rollout coordination already provided by `sdkwork-api-app-runtime`.

**Configuration Model**

Reuse `StandaloneConfigLoader` for API/runtime config.

Add product-host settings outside of `StandaloneConfig`:
- `SDKWORK_WEB_BIND`
- `SDKWORK_ROUTER_ROLES`
- `SDKWORK_ROUTER_NODE_ID_PREFIX`
- `SDKWORK_ADMIN_PROXY_TARGET`
- `SDKWORK_PORTAL_PROXY_TARGET`
- `SDKWORK_GATEWAY_PROXY_TARGET`

Desktop mode will not rely on these env vars directly; Tauri will construct product runtime options in code.

**Data Flow**

1. Load `StandaloneConfig`.
2. Start selected local services and capture actual binds.
3. Build an override config snapshot when ports were resolved dynamically.
4. Start service runtime supervision and rollout heartbeat tasks.
5. Start the public web host with local binds or explicit remote upstream targets.
6. In desktop mode, return the resolved public base URL to the frontend bridge.

**Packaging**

- Tauri bundles must include both:
  - portal `dist`
  - admin `dist`
- `beforeBuildCommand` must build both frontends before packaging.

**Testing**

- Add a product-runtime integration test that starts the unified host and verifies:
  - `/portal/`
  - `/admin/`
  - `/api/portal/health`
  - `/api/admin/health`
  - `/api/v1/health`
- Add runtime tests for ephemeral listener bind resolution.
- Add config-loader tests for override cloning.

**Non-Goals**

- Removing the existing standalone service binaries.
- Replacing the current routing/proxy behavior in `sdkwork-api-runtime-host`.
- Changing billing, token accounting, or database schemas in this iteration.
