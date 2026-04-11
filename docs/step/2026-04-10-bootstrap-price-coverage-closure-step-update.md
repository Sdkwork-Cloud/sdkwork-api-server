# 2026-04-10 Bootstrap Price Coverage Closure Step Update

## What Changed

- Hardened bootstrap validation so `request_meter_facts.provider_code + channel_code + model_code` must resolve to an active `model_prices/*` record.
- Hardened bootstrap validation so `billing_events.provider_id + channel_id + route_key` must resolve to an active `model_prices/*` record whenever the billing event declares a concrete channel route.
- Hardened bootstrap validation so every active `provider_models.proxy_provider_id + channel_id + model_id` tuple must resolve to an active `model_prices/*` record.
- Added regression tests for:
  - request metering evidence without active provider/channel/model price coverage
  - billing evidence without active provider/channel/route price coverage
  - active provider-model metadata without active provider/channel/model price coverage
- Repaired synthetic bootstrap fixtures so baseline commercial test packs include active proxy pricing for `provider-openrouter-main + openrouter + gpt-4.1`.

## Why This Matters

- Commercial execution evidence is not complete if it can name a provider/model tuple that has no billable price surface.
- Without this closure, seeded dev or prod environments can accept:
  - metering facts that cannot be priced into cost analytics
  - billing events that reference executable model routes with no active catalog pricing
- That creates silent drift between routing, observability, billing, and admin pricing views. The data may look structurally valid, but it is not commercially operable.

## Data Impact

- No repository `/data` seed pack required changes because current seed data already satisfies the stronger price-coverage closure.
- A repository audit confirmed:
  - `data/request-metering/*.json` already has active `provider + channel + model` price coverage
  - `data/billing/*.json` already has active `provider + channel + route_key` price coverage where channel-scoped billing evidence exists
  - `data/provider-models/*.json` already has active `provider + channel + model` price coverage for every active provider model
- Synthetic bootstrap fixtures were updated so unrelated validation tests keep failing for their intended reason instead of tripping over missing `openrouter::gpt-4.1` proxy pricing first.

## Verification

- `cargo test -p sdkwork-api-app-runtime active_model_price_coverage -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_active_provider_model_without_active_price_coverage -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime model_price_with_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime provider_channel_binding_without_active_provider_model_coverage -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- The next high-value commercial closure is price versioning and update semantics:
  - bind execution evidence to the active price revision window, not just the tuple key
  - define how future `/data/model-prices/*` updates supersede prior records without breaking idempotent bootstrap replay
