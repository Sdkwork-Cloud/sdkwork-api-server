# 2026-04-09 Runtime Governance Bootstrap Step Update

## Summary

This step extends repository bootstrap beyond catalog, pricing, routing, billing, and commerce so fresh `dev` and `prod` environments also start with plugin/runtime governance data.

The bootstrap framework now loads and applies three new `/data` domains:

- `service-runtime-nodes`
- `extension-runtime-rollouts`
- `standalone-config-rollouts`

Each domain is grouped under its own JSON directory, wired through ordered update manifests, validated for cross-domain references, and applied idempotently through the existing registry pipeline.

## What Changed

### Bootstrap Framework

- Extended `BootstrapDataPack` and `BootstrapBundleRefs` to support runtime governance domains.
- Added bundle loaders for:
  - extension rollout bundles
  - standalone config rollout bundles
- Added validation rules for:
  - runtime node identity and heartbeat timestamps
  - rollout scope correctness
  - extension id and extension instance references
  - rollout participant to runtime node references
  - participant `service_kind` consistency
  - rollout timestamp ordering
- Added registry stages for:
  - `service_runtime_nodes`
  - `extension_runtime_rollouts`
  - `extension_runtime_rollout_participants`
  - `standalone_config_rollouts`
  - `standalone_config_rollout_participants`

### Repository Data

- Added `data/updates/2026-04-runtime-governance-foundation.json`
- Added grouped seed files under:
  - `data/service-runtime-nodes/`
  - `data/extension-runtime-rollouts/`
  - `data/standalone-config-rollouts/`
- Wired the new update pack into:
  - `data/profiles/prod.json`
  - `data/profiles/dev.json`

### Test Coverage

- Added happy-path bootstrap assertions for runtime governance seed data.
- Added idempotency assertions so rerunning bootstrap does not duplicate governance records.
- Added a negative validation test for rollout participants referencing missing runtime nodes.
- Extended repository product-runtime assertions to verify the new governance seed pack for both `prod` and `dev`.

## Seed Data Design

The governance foundation pack deliberately covers multiple rollout patterns:

- instance-scoped extension rollout
- extension-scoped extension rollout
- all-scope extension resync
- service-scoped standalone config rollout
- global standalone config hydration rollout

This gives fresh installs realistic operational data for:

- runtime supervision dashboards
- rollout status pages
- plugin lifecycle verification
- deployment smoke tests

## Result

`dev` and `prod` bootstrap now deliver:

- richer provider and pricing readiness
- richer commerce and billing readiness
- richer async job readiness
- first-class runtime governance readiness

The initialization framework remains update-safe, additive, idempotent, and easy to extend with future domain packs.
