# 2026-04-10 Bootstrap Observability Price Readiness Step Update

## What Changed

- Hardened bootstrap validation so every `compiled_routing_snapshots.default_provider_id` must have active `model_prices/*` coverage for the snapshot `route_key` on at least one bound provider channel.
- Hardened bootstrap validation so every `routing_decision_logs.selected_provider_id` must have active `model_prices/*` coverage for the decision `route_key` on at least one bound provider channel.
- Added regression tests for:
  - compiled routing snapshots whose default provider is executable but not price-covered for the declared route
  - routing decision logs whose selected provider is executable but not price-covered for the declared route

## Why This Matters

- `default_provider_id` and `selected_provider_id` are not speculative metadata. They are the operational path the system will fall back to or has already chosen.
- If bootstrap accepts a default or selected provider without active route pricing, dev and prod seeds can look runnable while still lacking the commercial cost surface required by billing, margin analysis, and admin pricing views.
- This creates an especially dangerous gap: routing evidence says a route is viable, but the catalog cannot price it.

## Scope Boundary

- The repository audit confirmed that the stronger invariant is safe for:
  - `compiled_routing_snapshots.default_provider_id`
  - `routing_decision_logs.selected_provider_id`
- The same audit showed that it is **not** currently safe to enforce for:
  - all `compiled_routing_snapshots.ordered_provider_ids`
  - all `routing_decision_logs.assessments[*].provider_id`
- Those fields still contain strategy and comparison candidates in real `/data` that are intentionally broader than the currently active price surface. Enforcing price readiness there would reject real seed packs instead of exposing a true bootstrap contract.

## Data Impact

- No repository `/data` seed pack required changes because current seed data already satisfies the new default-provider and selected-provider price closure.
- Synthetic test fixtures also required no normalization:
  - the baseline compiled snapshot fixture already uses a price-covered `default_provider_id`
  - the baseline routing decision fixture already uses a price-covered `selected_provider_id`

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_compiled_snapshot_default_provider_without_active_price_coverage -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_decision_selected_provider_without_active_price_coverage -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- If the product later wants full candidate-level commercial readiness, the next step is not to add more validation first. It is to normalize `/data/observability/*` and related routing packs so speculative providers and comparison assessments are either:
  - backed by active route pricing, or
  - explicitly marked as non-billable comparison candidates in the data model
