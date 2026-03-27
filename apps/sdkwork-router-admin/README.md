# SDKWork Router Admin

`sdkwork-router-admin` is a standalone React workspace for the SDKWork Router super-admin surface.

## Goals

- independent engineering project under `apps/`
- product-grade super-admin UI for daily router operations
- follows `ARCHITECT.md` package ownership and composition rules
- ships as a standalone React workspace with its own `claw-studio`-aligned theme, routing shell, and package graph
- operates on live admin-control-plane data for day-to-day management, audit, and runtime posture

## Workspace Layout

```text
apps/sdkwork-router-admin/
├── src/        # root app shell composition only
├── packages/   # reusable foundation and business modules
├── tests/      # structure and architecture checks
└── dist/       # production build output
```

## Package Map

### Foundation

- `sdkwork-router-admin-types`
- `sdkwork-router-admin-commons`
- `sdkwork-router-admin-core`
- `sdkwork-router-admin-shell`
- `sdkwork-router-admin-admin-api`

### Business

- `sdkwork-router-admin-auth`
- `sdkwork-router-admin-overview`
- `sdkwork-router-admin-users`
- `sdkwork-router-admin-tenants`
- `sdkwork-router-admin-coupons`
- `sdkwork-router-admin-catalog`
- `sdkwork-router-admin-traffic`
- `sdkwork-router-admin-operations`
- `sdkwork-router-admin-settings`

## Data Sources

- live admin API:
  - operator users
  - portal users
  - tenants
  - projects
  - gateway keys
  - channels
  - providers
  - provider credentials
  - models
  - coupons
  - usage records
  - billing summary
  - routing decision logs
  - provider health
  - runtime status

## Product Surfaces

- `Overview`: global posture, operator alerts, top portal users, and hottest projects
- `Users`: operator and portal user CRUD, password rotation, status management, and usage visibility
- `Tenants`: tenant/project CRUD, gateway key issuance, revoke/restore, deletion, and workspace ownership checks
- `Coupons`: live campaign CRUD and activation posture
- `Catalog`: channel, proxy provider, provider credential, and model CRUD with credential coverage and secret rotation workflow
- `Traffic`: multi-filter request query console, CSV export, usage records, billing rollups, request-log visibility, user traffic leaderboard, and project hotspots
- `Operations`: provider health, runtime posture, and runtime reload controls
- `Settings`: theme mode, theme color, sidebar visibility, and shell posture controls

## Shell Model

- `react-router-dom` browser routes under `/admin/`
- claw-studio style shell with top header, left sidebar, and right content region
- click-to-collapse and resize-capable sidebar
- persistent theme mode and accent theme selection
- login kept outside the authenticated shell

## Commands

```bash
pnpm install
pnpm typecheck
pnpm build
pnpm dev
pnpm tauri:dev
pnpm tauri:build
```

The Vite dev server proxies `/api/admin/*` to `http://127.0.0.1:8081/admin/*`.

## Desktop Product Host

The admin desktop shell now uses the shared `sdkwork-api-product-runtime` instead of a hard-coded web host bootstrap.

That means the admin desktop app:

- starts the same shared router product runtime used by the portal desktop app
- serves both bundled admin and portal static sites
- exposes the same public web host contract as server mode
- keeps the existing admin IPC commands while resolving the runtime base URL at startup

The admin Tauri bundle includes:

- `embedded-sites/admin`
- `embedded-sites/portal`

This keeps the super-admin desktop experience aligned with the server-delivered `/admin/*` and `/portal/*` contract.

## Relationship To Server Mode

Server mode is owned by `apps/sdkwork-router-portal` through `pnpm server:start`, which launches `router-product-service`.

Use the admin app when you want:

- a local operator desktop shell
- bundled admin plus portal assets
- the full router product running on loopback for local operations

Use server mode when you want:

- browser access to `/admin/*` and `/portal/*`
- a deployable service process
- cluster role slicing across `web`, `gateway`, `admin`, and `portal`
