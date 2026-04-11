# 2026-04-10 Bootstrap Pricing Governance Validation Step Update

## What Changed

- Hardened bootstrap validation so every `request_meter_facts.cost_pricing_plan_id` must reference an existing `pricing_plan` that is currently `active`.
- Hardened bootstrap validation so every `request_meter_facts.retail_pricing_plan_id` must reference an existing `pricing_plan` that is currently `active`.
- Hardened bootstrap validation so every `pricing_rates.provider_code` must resolve to a real provider in the routing/catalog seed data.
- Hardened bootstrap validation so every `pricing_rates.provider_code + model_code` pair must resolve to a provider-backed model through either:
  - official model catalog ownership: `models.provider_id + models.external_name`
  - proxy provider model coverage: active `provider_models.proxy_provider_id + provider_models.model_id`
- Hardened bootstrap validation so every provider-scoped model rate with `capability_code` only passes when that provider/model actually declares the capability.
- Hardened bootstrap validation so `pricing_rates.model_code` cannot appear without `pricing_rates.provider_code`.
- Hardened bootstrap validation so every `pricing_rates.status = active` record must belong to a parent `pricing_plan` whose status is also `active`.
- Hardened bootstrap validation so `pricing_plans.effective_to_ms` can never be earlier than `pricing_plans.effective_from_ms`.
- Hardened bootstrap validation so every active provider-model `pricing_rate` must also have active `model_prices/*` coverage for that provider/model on at least one bound provider channel.
- Hardened bootstrap validation so every active provider-level fallback `pricing_rate` must belong to a provider that has at least one active `model_prices/*` record on one of its bound channels.
- Hardened bootstrap validation so every active provider-scoped `pricing_rate` must target a provider that still has at least one executable `provider_account` path to a live extension/runtime instance.
- Hardened bootstrap validation so every `model_prices.proxy_provider_id + channel_id` tuple must land on a real provider channel binding rather than inventing sell-side coverage on an unbound channel.
- Hardened bootstrap validation so every active `model_prices.proxy_provider_id` record must target a provider that still has at least one executable `provider_account` path to a live extension/runtime instance.
- Hardened bootstrap validation so every active provider-level fallback `pricing_rate.capability_code` must be backed by at least one active priced model on that provider whose catalog/provider-model capability actually matches the declared capability.

## Why This Matters

- `request_meter_facts` represent real execution and settlement evidence. If they point at inactive pricing plans, bootstrap can seed a system that looks billable but is already using retired or draft pricing posture.
- `pricing_rates` are the commercial overlay that admin, cost governance, and route economics rely on. If a rate names a provider or provider-model pair that the catalog cannot actually resolve, the price surface becomes internally contradictory:
  - routing and execution say one thing
  - pricing governance says another
- Provider-level fallback rates are only safe if they still describe a real priced execution surface. If a provider-level rate declares `embeddings` but the provider only has priced `responses` models, admin pricing, settlement posture, and route economics drift apart even though every individual record still looks syntactically valid.
- `model_prices` are the upstream price truth that powers route economics, admin pricing views, and request settlement evidence. If a provider can publish an active price on a channel it never bound, the system can look commercially ready for a route that is not actually deployable.
- Active upstream price cards are also an execution-readiness contract. If `model_prices/*` stays active after every executable provider account path is gone, pricing, admin selection, and route economics can still look healthy even though the provider is no longer callable.
- This closure makes the seed data safer for commercial deployment without forcing a destructive rewrite of the current additive `/data` packs.
- It also closes a pricing-operability gap where sell-side price cards could remain `active` for a provider that no longer has any executable account binding, which would make route economics look deployable while execution is already impossible.

## Scope Boundary

- The repository audit confirmed that it is safe to enforce `active` pricing plans for:
  - `request_meter_facts.cost_pricing_plan_id`
  - `request_meter_facts.retail_pricing_plan_id`
- The same audit confirmed that it is safe to enforce provider/model/catalog closure for:
  - `pricing_rates.provider_code`
  - `pricing_rates.provider_code + model_code`
  - provider-model-specific `pricing_rates.capability_code`
- The repository audit also confirmed that it is safe to enforce the following structural pricing invariants:
  - `pricing_rates.model_code -> pricing_rates.provider_code`
  - `active pricing_rate -> active pricing_plan`
  - `pricing_plans.effective_to_ms >= pricing_plans.effective_from_ms`
- The repository audit also confirmed that it is safe to enforce active upstream price readiness for:
  - `active pricing_rates.provider_code + model_code`
  - `active provider-level pricing_rates.provider_code`
- The repository audit also confirmed that it is safe to enforce executable provider-account readiness for:
  - `active pricing_rates.provider_code`
  - `active model_prices.proxy_provider_id`
- The repository audit also confirmed that it is safe to enforce channel-binding closure for:
  - `model_prices.proxy_provider_id + channel_id`
- The repository audit also confirmed that it is safe to enforce capability-aware provider fallback price readiness for:
  - `active pricing_rates.provider_code + capability_code`
- Current repository audit result:
  - active provider-model pricing rates without active model price coverage: `0`
  - active provider-level fallback pricing rates without any active model price coverage: `0`
  - active provider-scoped pricing rates without executable provider-account coverage: `0`
  - active model price records without executable provider-account coverage: `0`
  - model price records without provider channel binding coverage: `0`
  - active provider-level pricing rates without a capability-matched active priced model: `0`
- This update intentionally does **not** enforce pricing-plan effective window coverage for request timestamps.
  - real `/data` still contains additive update packs that relink older requests onto newer pricing plans
  - enforcing `started_at_ms` inside `effective_from_ms/effective_to_ms` would currently reject valid repository seed packs
- This update also does **not** force provider-only fallback rates to prove capability coverage for every possible model.
  - current provider-level rates are still posture-level defaults, not exact execution contracts

## Data Impact

- No repository `/data` seed pack required changes.
- Real `data/pricing/*.json`, `data/request-metering/*.json`, and additive update packs already satisfy the new bootstrap rules.
- Real `data/model-prices/*.json` and merged profile update packs also already keep every price record on a channel that the provider actually binds.
- The new validation therefore increases correctness without creating migration churn or breaking idempotent bootstrap behavior.
- Regression fixtures that intentionally remove a provider account now also rewrite unrelated `model_prices/*` coverage when needed so older tests still fail on their intended routing/pricing invariant instead of a newer executable-account guard.

## Test Coverage Added

- request meter fact rejects inactive cost pricing plan linkage
- request meter fact rejects inactive retail pricing plan linkage
- pricing rate rejects missing provider reference
- pricing rate rejects provider/model pairs not backed by catalog/provider-model data
- pricing rate rejects provider-model capability declarations that are not actually supported
- pricing rate rejects `model_code` without `provider_code`
- pricing rate rejects `active` status when parent pricing plan is not `active`
- pricing plan rejects reversed effective window
- pricing rate rejects active provider-model pricing without active model price coverage
- pricing rate rejects active provider fallback pricing without any active model price coverage
- pricing rate rejects active provider-scoped pricing when the provider has no executable provider account
- model price rejects provider/channel combinations that are outside provider bindings
- model price rejects active proxy-provider pricing when the provider has no executable provider account
- pricing rate rejects active provider fallback pricing when no active priced model matches the declared capability

## Verification

- `cargo test -p sdkwork-api-app-runtime inactive_cost_pricing_plan -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime inactive_retail_pricing_plan -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime pricing_rate_with_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime model_code_without_provider_code -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime inactive_parent_plan -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime effective_to_before_effective_from -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime active_pricing_rate_without_active_model_price_coverage -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime active_provider_rate_without_any_active_model_price_coverage -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime active_provider_rate_without_executable_provider_account -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime model_price_channel_outside_provider_bindings -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime active_model_price_without_executable_provider_account -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime active_provider_rate_without_capability_matched_active_model_price_coverage -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Tooling Note

- A separate compile blocker in `crates/sdkwork-api-domain-marketing` was preventing Rust verification from starting:
  - source code already used `anyhow`
  - the crate manifest did not declare `anyhow.workspace = true`
- The fix was limited to adding the missing dependency declaration so bootstrap verification could proceed.
- A second independent compile blocker surfaced during product runtime verification in `crates/sdkwork-api-interface-admin/src/marketing.rs`:
  - new lifecycle handlers used `Extension<RequestId>` without importing `axum::Extension`
  - lifecycle mutation code moved canonical records before building audit evidence
- The fix was limited to:
  - importing `axum::Extension`
  - cloning the pre-mutation records before consuming builder-style `with_status(...)`
- A later verification round surfaced matching compile blockers in both marketing storage backends:
  - `crates/sdkwork-api-storage-sqlite/src/marketing_store_impl.rs`
  - `crates/sdkwork-api-storage-postgres/src/marketing_store_impl.rs`
- Both files referenced new lifecycle audit record types without importing them.
- The fix was limited to adding the missing `sdkwork_api_domain_marketing::{CampaignBudgetLifecycleAuditRecord, CouponCodeLifecycleAuditRecord}` imports.

## Follow-Up

- If the product later needs full historical pricing audit correctness, first normalize `/data/request-metering/*` and additive pricing updates so request timestamps and plan effective windows align. Only then add effective-window validation.
- If admin needs stronger sell-side governance, the next safe step is to add read models that join:
  - `pricing_plan`
  - `pricing_rate`
  - `model_price`
  - `provider_models`
  - `request_meter_facts`
  rather than overloading bootstrap validation with reporting concerns.
- If the product later introduces draft or scheduled provider-level commercial rates for providers that are not yet price-curated, keep the active-only boundary.
  - `draft` and `planned` style future records should remain creatable without forcing immediate `model_prices/*` closure
  - the stronger requirement should continue to apply only when a rate becomes operationally `active`
