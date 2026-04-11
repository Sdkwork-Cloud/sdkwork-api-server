# Bootstrap Provider Operations Readiness Step Update

## What changed

- added `data/updates/2026-04-global-provider-operations-readiness.json`
- added grouped bootstrap data for:
  - `data/routing/2026-04-global-provider-operations-readiness.json`
  - `data/api-key-groups/2026-04-global-provider-operations-readiness.json`
  - `data/quota-policies/2026-04-global-provider-operations-readiness.json`
  - `data/observability/2026-04-global-provider-operations-readiness.json`
  - `data/billing/2026-04-global-provider-operations-readiness.json`
  - `data/pricing/2026-04-global-provider-operations-readiness.json`
  - `data/payment-methods/2026-04-global-provider-operations-readiness.json`
  - `data/marketing/2026-04-global-provider-operations-readiness.json`
  - `data/commerce/2026-04-global-provider-operations-readiness.json`
  - `data/jobs/2026-04-global-provider-operations-readiness.json`
- wired the update into both repository profiles:
  - `data/profiles/prod.json`
  - `data/profiles/dev.json`

## Why this pack exists

The earlier catalog and official-route packs made the system runnable and channel-complete. This pack moves the bootstrap experience closer to install-time commercial operation by filling the remaining operator surfaces:

- provider-specific API key groups
- proxy and local execution profiles
- pricing plans and pricing rates aligned to execution providers
- payment-method diversity
- coupon, campaign, and budget samples
- commerce settlement and reconciliation samples
- async job samples across official, proxy, and local execution paths

## Design constraints

- bootstrap still stays idempotent because every record uses stable IDs
- update order stays explicit through `depends_on`
- provider-specific route samples only reference models that already have declared `provider-model` coverage
- payment and commerce records stay synthetic and safe for repository storage
- the pack expands operational realism without introducing secrets or environment-specific credentials

## Outcome

After this pack is applied, a fresh dev or prod deployment has a much richer default control-plane surface for:

- routing governance
- provider-specific usage segmentation
- commercial pricing and checkout workflows
- campaign and coupon management
- finance reconciliation workflows
- async job visibility across official, proxy, and local providers
