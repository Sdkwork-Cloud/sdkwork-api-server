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
```

The Vite dev server proxies `/api/portal/*` to `http://127.0.0.1:8082/portal/*`.
