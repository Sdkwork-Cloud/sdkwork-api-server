# 2026-04-10 Bootstrap Provider Metering Capability Closure Step Update

## What Changed

- Hardened bootstrap validation so `request_meter_facts.capability_code` must be supported not only by the `channel_code + model_code` channel model, but also by the effective provider-side model metadata.
- Provider-side capability coverage now resolves through either:
  - direct provider model metadata in `models/*`
  - proxy provider model metadata in `provider-models/*`
- Added a regression test for request metering evidence where the channel model still supports the capability but the provider model does not.

## Why This Matters

- Channel-model capability coverage is not enough for commercial correctness.
- A proxy provider may expose only a subset of a canonical channel model's capabilities, or may publish provider-specific execution models with narrower support.
- If bootstrap only checks channel-model capability, seeded environments can still ingest metering evidence that the chosen provider could not have executed under its own model contract.

## Data Impact

- No repository `/data` seed pack required changes.
- A direct audit confirmed there is no provider-specific capability drift across `data/request-metering/*.json`.
- Existing commercial seed packs already satisfy the stronger closure for:
  - official direct providers
  - OpenRouter proxy routes
  - SiliconFlow proxy routes
  - Ollama local routes

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_request_meter_fact_with_capability_not_supported_by_provider_model -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_request_meter_fact_with_capability_not_supported_by_channel_model -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- The next high-value closure candidate is pricing semantic alignment:
  - bind `request_meter_facts.capability_code + model_code + provider_code` to an active, capability-compatible pricing surface
  - ensure seeded execution evidence cannot reference a provider/model/capability tuple that has no commercially billable price coverage
