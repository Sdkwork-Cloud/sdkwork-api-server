# 2026-04-10 Bootstrap Routing Policy Viability Step Update

## What Changed

- Hardened bootstrap validation so every enabled `routing_policy` with declared providers must have at least one declared provider that is commercially executable for the policy itself.
- The new routing viability check requires one declared provider to satisfy all of the following at the same time:
  - the provider has an enabled executable account path
  - the provider has at least one active `model_prices/*` record
  - the priced model matches `routing_policies.model_pattern`
  - the priced model is capability-compatible with `routing_policies.capability`
- The check works across both:
  - official catalog ownership: `models.provider_id + models.external_name + capabilities`
  - proxy catalog coverage: active `provider_models.proxy_provider_id + model_id + capabilities`

## Why This Matters

- Before this change, bootstrap could accept an enabled routing policy that looked structurally valid but was operationally dead.
- That meant admin and route configuration could declare a policy for `capability + model_pattern` even when none of the policy's declared providers had any active priced model that could actually satisfy the route.
- Runtime would eventually fall back, but the seed data itself would still advertise a route policy that was not commercially deployable.
- This closes the gap earlier, at bootstrap time, without over-constraining legitimate broad fallback chains.

## Scope Boundary

- The repository audit confirmed it is safe to enforce:
  - enabled `routing_policies` must have at least one declared provider with capability-matched active model price coverage
- The repository audit also confirmed it is **not** yet safe to enforce the stronger rule:
  - every declared provider on every enabled `routing_policy` must individually satisfy the same capability-matched active price coverage
- Current repository audit result:
  - enabled routing policies with zero declared-provider capability-matched active model price coverage in `prod`: `0`
  - enabled routing policies with zero declared-provider capability-matched active model price coverage in `dev`: `0`
  - declared routing-policy providers without individual capability-matched active model price coverage in `prod`: `6`
  - declared routing-policy providers without individual capability-matched active model price coverage in `dev`: `6`
- Real examples that currently block the stronger per-provider invariant:
  - `policy-asia-official-responses` declares fallback providers that do not individually match `ernie-*`
  - `policy-catalog-qwen-responses` and `policy-catalog-local-qwen-responses` still include broader fallback providers whose active priced models do not exactly match the policy pattern

## Data Impact

- No repository `/data` seed pack required changes.
- Real `data/routing/*.json`, `data/model-prices/*.json`, `data/models/*.json`, and `data/provider-models/*.json` already satisfy the new viability floor for both `prod` and `dev`.
- The new validation therefore increases routing correctness without introducing migration churn or breaking idempotent bootstrap behavior.

## Test Coverage Added

- routing policy rejects enabled configuration when none of its declared providers has any capability-matched active model price coverage

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_enabled_routing_policy_without_any_capability_matched_active_model_price_coverage -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- If the product later wants stricter route-governance, the next candidate step is to normalize the handful of broad fallback policies that currently declare providers without exact pattern coverage.
- Only after that data normalization is complete should bootstrap enforce the stronger rule that every declared provider on an enabled policy must individually satisfy capability-matched active price coverage.
