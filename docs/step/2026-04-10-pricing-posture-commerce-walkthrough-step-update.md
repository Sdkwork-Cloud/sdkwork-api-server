# 2026-04-10 Pricing Posture Commerce Walkthrough Step Update

## Completed

### Step 1 - Added a dedicated additive walkthrough pack

- added [`2026-04-global-pricing-posture-commerce-walkthrough.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/updates/2026-04-global-pricing-posture-commerce-walkthrough.json)
- kept the change inside the existing bootstrap pack system:
  - no new runtime bootstrap path
  - no baseline rewrite
  - no special-case importer branch

### Step 2 - Expanded commercial-ready marketing data

- added new marketing seeds in [`2026-04-global-pricing-posture-commerce-walkthrough.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/marketing/2026-04-global-pricing-posture-commerce-walkthrough.json)
- added official-direct and local-edge campaign scaffolding:
  - `official-direct-credit-100`
  - `edge-local-credit-20`
  - `campaign-official-direct-2026`
  - `campaign-edge-local-2026`
- kept the data operator-friendly for admin and portal walkthroughs:
  - legacy coupon summary rows
  - canonical coupon templates
  - campaign budgets
  - claimable coupon codes

### Step 3 - Added richer commerce walkthrough data

- added new commerce seeds in [`2026-04-global-pricing-posture-commerce-walkthrough.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/commerce/2026-04-global-pricing-posture-commerce-walkthrough.json)
- added a completed official-direct sample:
  - `order-global-official-openai-2026`
  - `attempt-global-official-openai-2026`
  - `payment-event-global-official-openai-2026`
  - `webhook-inbox-global-official-openai-2026`
  - `refund-global-official-openai-2026`
  - `recon-run-global-official-openai-2026`
- added a completed local-edge sample:
  - `order-edge-local-ollama-2026`
  - `attempt-edge-local-ollama-2026`
  - `payment-event-edge-local-ollama-2026`
  - `recon-run-edge-local-ollama-2026`
  - `recon-item-edge-local-ollama-2026`

### Step 4 - Bound walkthroughs to the pricing posture model

- official-direct walkthrough order now references retail pricing posture `global-official-direct-retail`
- local-edge walkthrough order now references retail pricing posture `edge-local-commercial`
- the richer commercial chain stays consistent with the established pricing hierarchy:
  - `model-price` = exact provider-model reference pricing
  - `pricing-plan` / `pricing-rate` = cost or sell-side posture
  - commerce order snapshot = operator-facing packaged offer context

### Step 5 - Kept dev and prod aligned

- added the new update pack to:
  - [`prod.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/profiles/prod.json)
  - [`dev.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/profiles/dev.json)
- this keeps development installs and production installs on the same additive commercial baseline while still allowing dev-only packs to extend on top

## Why this design

- the existing schema was already sufficient:
  - `channel-model` handles the canonical catalog
  - `provider-model` handles provider subset coverage
  - `route config` chooses providers against that subset
  - pricing posture classes already explain official, proxy, and local cost versus retail
- the missing piece was install-ready business walkthrough data, not another schema redesign
- packaging the new flows as one additive update pack preserves:
  - high cohesion
  - low coupling
  - idempotent replay
  - safe future updates through last-wins stable-key merge

## Verified

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_discovers_repository_dev_bootstrap_profile_data_pack -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime product_runtime_bootstraps_repository_default_data_pack -- --nocapture`

## Next

- add more regional walkthrough packs the same way instead of expanding baseline files in place
- keep admin-side provider model management focused on explicit `provider-model` subsets rather than assuming full proxy coverage per channel
- if pricing views need stronger operator ergonomics, add read-model summaries that join:
  - `channel-model`
  - `provider-model`
  - `model-price`
  - `pricing-plan`
  - commerce walkthrough orders
