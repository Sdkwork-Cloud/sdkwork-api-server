# 2026-04-09 Bootstrap Catalog / Pricing Step Update

## Completed

### Step 1 - Canonical catalog semantics

- locked the bootstrap catalog boundary to:
  - `channel = canonical model inventor / vendor`
  - `provider = official, proxy, or local execution entry`
  - `channel-model = canonical public model identity`
  - `provider-model = canonical-to-provider model mapping`
  - `model-price = canonical model plus execution-provider price contract`
- kept route config provider-centric, with runtime resolving canonical model selection through `provider-model`
- preserved provider-specific `provider_model_id` rewriting at execution time so canonical routing and provider-native execution stay separated cleanly

### Step 2 - Idempotent bootstrap and update-pack layering

- repository bootstrap now loads domain-grouped `/data/*` JSON plus ordered `/data/updates/*.json`
- update packs reuse stable IDs and upsert-oriented stages, so repeated startup does not create duplicate dirty data
- prod and dev profiles both materialize repository-backed catalog data, while dev can safely overlay richer workspace/demo scaffolding

### Step 3 - Provider-model subset governance

- official channels seed their official providers by default
- proxy providers seed explicit `provider-model` subsets instead of implying full channel coverage
- bootstrap now carries default support mappings for:
  - official providers
  - OpenRouter
  - SiliconFlow
  - Ollama
- provider-model records own:
  - provider-native model id / family
  - capabilities
  - cache support
  - reasoning/tool-usage flags
  - default-route posture
  - active posture

### Step 4 - Friendly pricing contract

- kept normalized price columns for routing/accounting compatibility:
  - `input_price`
  - `output_price`
  - `cache_read_price`
  - `cache_write_price`
  - `request_price`
- added operator-friendly pricing metadata:
  - `price_source_kind`
  - `billing_notes`
  - `pricing_tiers`
- current `price_source_kind` values are:
  - `official`
  - `proxy`
  - `local`
  - `reference`
- `pricing_tiers` now covers:
  - prompt-length breakpoints
  - modality variants
  - cache-window variants
  - cache-storage pricing
  - request-level overrides

### Step 5 - Admin pricing governance surface

- admin TypeScript contract now exposes `ModelPriceTier` and the richer `ModelPriceRecord`
- admin API save payload now accepts:
  - `price_source_kind`
  - `billing_notes`
  - `pricing_tiers`
- catalog price dialog now supports:
  - price-source selection
  - billing-notes editing
  - pricing-tiers JSON authoring
- catalog detail panel now shows:
  - official/proxy/local price-source posture
  - flat versus tiered pricing summary
  - billing notes
  - tier-level price breakdown

## Verified

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_discovers_repository_bootstrap_profile_data_pack -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_discovers_repository_dev_bootstrap_profile_data_pack -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime product_runtime_bootstraps_repository_default_data_pack -- --nocapture`
- `node --test apps/sdkwork-router-admin/tests/admin-catalog-pricing-contract.test.mjs`
- `pnpm --dir apps/sdkwork-router-admin typecheck`

## Next

- keep bootstrap price rows aligned to upstream official/proxy price changes through new update packs instead of mutating baseline JSON in place
- add additional admin validation around malformed `pricing_tiers` JSON if operators need field-level inline error presentation
- keep proxy-provider onboarding model-subset based; do not widen proxy providers into implicit full-channel support
