# 2026-04-10 Bootstrap Billing Catalog Closure Step Update

## Summary

This step closes another commercial bootstrap data gap:

- `billing_events` previously required provider, tenant, project, route context, and snapshot references
- but they did not need to resolve back into the catalog model graph that pricing and admin explanations depend on

That meant a billing event could be structurally valid while still carrying a route key or usage model that the seeded catalog could not explain.

## What Changed

### 1. Added billing route-key to channel-model validation

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- when `billing_events.channel_id` is present, `billing_events.route_key` must resolve to the merged channel-model catalog for that channel

This ensures billing evidence points back to a real canonical route model in the initialized data set.

### 2. Added billing usage-model to provider mapping validation

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- a billing event must be explainable by either:
  - a direct provider model variant where `route_key == usage_model`, or
  - an active `provider_model` mapping for the same provider/channel where:
    - `provider_models.model_id == billing_events.route_key`
    - and `provider_models.provider_model_id == billing_events.usage_model`
      or `provider_models.model_id == billing_events.usage_model`

This keeps billing records aligned with the intended two-layer model:

- `route_key` is the canonical routed model
- `usage_model` is the provider-facing execution model identifier

That matters for commercial reasoning because catalog display, route explanations, and pricing all depend on this mapping being coherent.

### 3. Hardened regression coverage

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Added regression tests for:

- billing events whose `route_key` is missing from the channel-model catalog
- billing events whose `usage_model` is not mapped for the provider route

Also adjusted the local bootstrap test pack so its OpenRouter billing sample has explicit catalog coverage for `gpt-4.1`, keeping the default success pack semantically consistent under the stricter contract.

### 4. Verified merged production/bootstrap data still passes

No shipped `/data` file needed to change in this step.

The merged `prod` / `dev` bootstrap profiles already carry the required expansion packs for Asia providers, OpenRouter channel coverage, and provider-model mappings, so the stronger billing catalog contract holds after profile updates are applied.

## Verification

Passed:

- `cargo test -p sdkwork-api-app-runtime billing_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_loads_bootstrap_profile_data_pack -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Result

Bootstrap billing evidence is now materially stronger:

- every billed route can be traced back to a seeded canonical channel model
- every billed usage model must be explainable by the provider's seeded model mapping
- admin pricing, route diagnostics, and billing narratives now share a tighter common catalog contract

This reduces the chance of shipping a commercially rich `/data` pack that looks complete in the UI but cannot actually explain its own billing evidence.
