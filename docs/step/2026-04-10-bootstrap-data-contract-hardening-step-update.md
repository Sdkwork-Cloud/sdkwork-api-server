# 2026-04-10 Bootstrap Data Contract Hardening Step Update

## Summary

This step hardens the bootstrap data contract so the shipped `data/` packs are closer to commercial-ready runtime quality instead of only being referentially valid.

The work focuses on three gaps:

1. non-primary provider `channel_binding` rows could exist without real model or price coverage
2. routing defaults could point at providers that ship no enabled provider-account
3. model pricing records could carry weak or malformed pricing semantics

## What Changed

### 1. Added reverse coverage validation for provider channel bindings

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- if a provider declares a non-primary channel binding, that `(provider, channel)` pair must have at least one active `provider-model`
- the same `(provider, channel)` pair must also have at least one active `model-price`

This closes the gap where a proxy or marketplace provider could advertise channel support in `/data/providers/*.json` but still ship an unusable bootstrap route.

### 2. Added executable routing-default validation

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- `routing_profiles.default_provider_id` must resolve to a provider with at least one enabled `provider-account`
- `routing_policies.default_provider_id` must resolve to a provider with at least one enabled `provider-account`
- `project_preferences.default_provider_id` must resolve to a provider with at least one enabled `provider-account`

This keeps shipped route defaults executable on day 0 after deployment.

### 3. Hardened model-price semantics

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New checks:

- `price_source_kind` must be one of `official`, `proxy`, `local`, `reference`
- top-level price numbers must be finite and non-negative
- pricing tier ids must be unique per model-price record
- pricing tier ids, condition kinds, currency codes, and price units must be non-empty
- pricing tier price values must be finite and non-negative
- tier token ranges must not invert `min_input_tokens` and `max_input_tokens`

This makes pricing data safer for admin display, commercial quoting, and downstream pricing analysis.

### 4. Filled the real repository data gap for expansion providers

Updated:

- `data/updates/2026-04-global-catalog-expansion.json`
- `data/provider-accounts/2026-04-global-expansion.json`

Added shipped bootstrap accounts for:

- `provider-ernie-official`
- `provider-minimax-official`

These providers were already introduced by the global catalog expansion and used by default routes, but they were not yet backed by default official provider-accounts.

### 5. Extended bootstrap regression coverage

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Added regression tests for:

- non-primary provider channel binding without active provider-model coverage
- non-primary provider channel binding without active model-price coverage
- routing default provider without enabled provider-account
- unsupported `price_source_kind`
- duplicate pricing tier ids
- negative pricing tier values
- repository prod/dev bootstrap data now asserting ERNIE and MiniMax provider-account presence

Also updated the ordered-update-pack test fixture so its expansion data now includes:

- extension installations and instances for ERNIE and MiniMax
- provider-models and provider-accounts for those expansion providers

## Verification

Passed:

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_discovers_repository_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Result

The bootstrap system now enforces a stricter commercial-readiness floor:

- declared channel coverage must be real
- default routing must be executable
- pricing records must be structurally sane
- shipped expansion providers must include runnable default accounts

This makes future `/data` update packs safer to compose and reduces the chance of shipping route-visible but non-executable catalog data.
