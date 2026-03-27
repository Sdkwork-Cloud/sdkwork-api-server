# SDKWork Router Portal

`sdkwork-router-portal` is the standalone developer self-service workspace for SDKWork Router.

## Goals

- independent engineering project under `apps/`
- product-grade self-service portal rather than a legacy `console` sub-page
- follows `ARCHITECT.md` package ownership and dependency direction
- combines live `/portal/*` data with explicit commerce repository seams where backend payment
  integration is not ready yet

## Workspace Layout

```text
apps/sdkwork-router-portal/
├── src/        # root shell composition and theme only
├── packages/   # foundation and business modules
├── tests/      # structure and architecture checks
└── dist/       # production build output
```

## Business Package Standard

Portal business packages follow the `ARCHITECT.md` shape:

```text
packages/sdkwork-router-portal-<module>/src/
├── types/       # local view contracts and page props
├── components/  # module-owned UI pieces
├── repository/  # live portal API or commerce seam access
├── services/    # derived product logic and recommendations
├── pages/       # route-level page entry
└── index.tsx    # thin re-export surface
```

## Package Map

### Foundation

- `sdkwork-router-portal-types`
- `sdkwork-router-portal-commons`
- `sdkwork-router-portal-core`
- `sdkwork-router-portal-portal-api`
- `sdkwork-router-portal-commerce`

### Business

- `sdkwork-router-portal-auth`
- `sdkwork-router-portal-dashboard`
- `sdkwork-router-portal-routing`
- `sdkwork-router-portal-api-keys`
- `sdkwork-router-portal-usage`
- `sdkwork-router-portal-user`
- `sdkwork-router-portal-credits`
- `sdkwork-router-portal-billing`
- `sdkwork-router-portal-account`

## Product Surface

`User` handles profile and password rotation.
`Account` handles cash balance, credits, billing ledger, and runway posture.

- `Dashboard`
  - workspace identity
  - points and quota posture
  - total requests
  - token-unit usage
  - recent request list
  - recommended next actions derived from live posture
- `Routing`
  - active routing strategy and preset
  - provider ordering and healthy-path guardrails
  - preview route simulation
  - routing decision evidence
- `API Keys`
  - environment-scoped key issuance
  - plaintext-once handling
  - environment coverage summaries
  - copy and quickstart handoff
- `Usage`
  - request telemetry
  - model and provider distribution
  - per-call token-unit history
  - input, output, and total token detail
  - filterable workbench for models, providers, and time ranges
- `User`
  - profile and password rotation
  - personal security posture
  - recovery guidance
- `Credits`
  - quota posture
  - points-oriented ledger view
  - coupon redemption seam
  - redemption-impact preview
- `Billing`
  - subscription plan catalog
  - recharge packs
  - upgrade motion
  - recommendation logic based on current usage and remaining quota
- `Account`
  - cash balance
  - credits and quota posture
  - billing ledger
  - runway posture

## Data Sources

- live portal API:
  - auth session
  - workspace summary
  - dashboard summary
  - routing summary, preferences, preview, and decision logs
  - usage records and summary
  - billing summary and ledger
  - API keys
  - password rotation
- workspace-scoped commerce repository seam:
  - subscription plans
  - recharge packs
  - coupon catalog

## Commands

```bash
pnpm install
pnpm typecheck
pnpm build
pnpm preview
pnpm dev
pnpm product:start
pnpm server:start
pnpm server:plan
pnpm product:check
pnpm tauri:dev
pnpm tauri:build
```

The Vite dev server proxies `/api/portal/*` to `http://127.0.0.1:8082/portal/*`.

## Product Runtime Modes

`sdkwork-router-portal` is now the product entrypoint for both desktop mode and server mode.

- desktop mode uses the Tauri shell in `src-tauri/`
- server mode starts `router-product-service`
- `pnpm product:start` is the unified launcher for desktop, server, plan, and check workflows
- both modes use the shared `sdkwork-api-product-runtime`
- both modes serve the public web host plus router APIs together

### Desktop Mode

Desktop mode starts the full router product locally:

- embedded web host binds to `127.0.0.1:0`
- gateway, admin, and portal services bind to loopback ephemeral ports
- both `sdkwork-router-portal` and `sdkwork-router-admin` static bundles are packaged into the app
- the desktop shell resolves the runtime base URL dynamically through Tauri IPC

Useful commands:

```bash
pnpm product:start
pnpm product:start -- desktop
pnpm desktop:build-assets
pnpm tauri:dev
pnpm tauri:build
```

### Server Mode

`pnpm server:start` runs `router-product-service`, which starts:

- the public web host
- the gateway API
- the admin API
- the portal API

By default, server mode exposes:

- `/portal/*` for the portal web app
- `/admin/*` for the super-admin web app
- `/api/portal/*` for portal APIs
- `/api/admin/*` for admin APIs
- `/api/v1/*` for OpenAI-compatible gateway APIs
- `/api/v1/health` and `/api/v1/metrics` through the gateway proxy

Default public bind:

```bash
SDKWORK_WEB_BIND=0.0.0.0:3001
```

Server mode also supports direct CLI flags, which override environment variables:

```bash
pnpm product:start -- server
pnpm server:start -- --help
pnpm product:start -- plan
pnpm product:start -- check
pnpm server:plan
pnpm product:check
pnpm server:start -- --bind 0.0.0.0:3301 --roles web,gateway,admin,portal
pnpm server:start -- --database-url postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router
pnpm server:start -- --dry-run --roles web --gateway-upstream 10.0.0.21:8080 --admin-upstream 10.0.0.22:8081 --portal-upstream 10.0.0.23:8082
```

Common CLI flags:

- `--config-dir`
- `--config-file`
- `--database-url`
- `--bind`
- `--roles`
- `--node-id-prefix`
- `--gateway-bind`
- `--admin-bind`
- `--portal-bind`
- `--gateway-upstream`
- `--admin-upstream`
- `--portal-upstream`
- `--admin-site-dir`
- `--portal-site-dir`
- `--dry-run`

`--dry-run` prints the resolved deployment plan and exits without binding ports. It is intended
for cluster rollout review, container entrypoint validation, and CI smoke checks.

`pnpm product:start` defaults to desktop mode and can also launch `server`, `plan`, `check`, and
`browser` modes through one product-level entrypoint:

```bash
pnpm product:start
pnpm product:start -- server --roles web --gateway-upstream 10.0.0.21:8080
pnpm product:start -- plan --roles web
pnpm product:start -- check
```

When a forwarded mode argument would conflict with the launcher itself, use a second `--`:

```bash
pnpm product:start -- server -- --help
```

`pnpm server:plan` emits the dry-run plan as JSON so deployment tooling can inspect the resolved
roles, binds, upstreams, database URL, and bundled web assets.

`pnpm product:check` runs the integrated portal/admin typechecks, rebuilds bundled desktop assets,
checks `router-product-service`, and finishes with a JSON dry-run plan.

### Cluster and Role Slicing

Server mode supports role-based topology so the product can scale beyond a single process.

Supported roles:

- `web`
- `gateway`
- `admin`
- `portal`

Use `SDKWORK_ROUTER_ROLES` to select which roles a node owns:

```bash
SDKWORK_ROUTER_ROLES=web,gateway,admin,portal
```

When `web` is enabled without a local API role, the matching upstream must be configured:

- `SDKWORK_GATEWAY_PROXY_TARGET`
- `SDKWORK_ADMIN_PROXY_TARGET`
- `SDKWORK_PORTAL_PROXY_TARGET`

Example edge-only node:

```bash
pnpm server:start -- \
  --bind 0.0.0.0:3001 \
  --roles web \
  --gateway-upstream 10.0.0.21:8080 \
  --admin-upstream 10.0.0.22:8081 \
  --portal-upstream 10.0.0.23:8082
```

Example edge-only dry-run validation:

```bash
pnpm server:start -- \
  --dry-run \
  --roles web \
  --bind 0.0.0.0:3001 \
  --gateway-upstream 10.0.0.21:8080 \
  --admin-upstream 10.0.0.22:8081 \
  --portal-upstream 10.0.0.23:8082
```

Example split control-plane node:

```bash
pnpm server:start -- \
  --roles admin,portal \
  --node-id-prefix control-a \
  --admin-bind 127.0.0.1:9081 \
  --portal-bind 127.0.0.1:9082
```

Example split data-plane node:

```bash
pnpm server:start -- \
  --roles gateway \
  --node-id-prefix gateway-a \
  --gateway-bind 127.0.0.1:9080
```

### Runtime Environment

The shared product runtime recognizes these environment variables:

- `SDKWORK_WEB_BIND`
- `SDKWORK_DATABASE_URL`
- `SDKWORK_ROUTER_ROLES`
- `SDKWORK_ROUTER_NODE_ID_PREFIX`
- `SDKWORK_GATEWAY_PROXY_TARGET`
- `SDKWORK_ADMIN_PROXY_TARGET`
- `SDKWORK_PORTAL_PROXY_TARGET`
- `SDKWORK_ADMIN_SITE_DIR`
- `SDKWORK_PORTAL_SITE_DIR`

The rest of the standalone router configuration still comes from `sdkwork-api-config`.
