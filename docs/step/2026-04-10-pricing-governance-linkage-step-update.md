# 2026-04-10 Pricing Governance Linkage Step Update

## Completed

### Step 1 - Locked the pricing hierarchy boundary

- kept the existing normalized pricing split:
  - `model-price` = canonical channel-model plus execution-provider reference price
  - `pricing-plan` / `pricing-rate` = internal cost or sell-side commercial class
  - `request-meter-fact` = request-level linkage to both cost and retail pricing posture
- clarified the design rule:
  - exact provider-model numbers stay on `model-price`
  - billing posture lives on plan/rate classes
  - requests point at both through `cost_pricing_plan_id` and `retail_pricing_plan_id`

### Step 2 - Added additive pricing-governance update packs

- added [`2026-04-global-pricing-governance-linkage.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/updates/2026-04-global-pricing-governance-linkage.json)
- added [`2026-04-dev-pricing-governance-linkage.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/updates/2026-04-dev-pricing-governance-linkage.json)
- kept the change additive only:
  - no baseline rewrite
  - no special-case bootstrap path
  - no account-kernel record count changes

### Step 3 - Introduced explicit pricing posture classes

- added new shared pricing plans in [`2026-04-global-pricing-governance-linkage.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/pricing/2026-04-global-pricing-governance-linkage.json):
  - `global-official-direct-cost`
  - `global-official-direct-retail`
  - `china-official-direct-cost`
  - `china-official-direct-retail`
  - `global-marketplace-proxy-cost`
  - `china-proxy-distribution-cost`
  - `local-edge-infra-cost`
- reused existing commercial retail plans where they already matched the operating posture:
  - `global-provider-mix-commercial`
  - `china-direct-commercial`
  - `edge-local-commercial`
- added provider-level fallback rates so admin and billing surfaces show meaningful defaults even when the exact model-specific commercial override is not yet curated.

### Step 4 - Relinked request metering to cost and retail plans

- prod request facts now resolve into explicit pricing classes:
  - official global direct traffic -> `9106` cost / `9107` retail
  - official China direct traffic -> `9108` cost / `9109` retail
  - OpenRouter marketplace traffic -> `9110` cost / `9103` retail
  - SiliconFlow distribution traffic -> `9111` cost / `9104` retail
  - Ollama local edge traffic -> `9112` cost / `9105` retail
- dev-only request facts now reuse the same price classes:
  - local sandbox Ollama -> `9112` / `9105`
  - partner Gemini official -> `9106` / `9107`
  - growth lab Minimax official -> `9108` / `9109`

### Step 5 - Preserved idempotent update behavior

- request-metering relink is implemented as last-wins stable-key overrides on `request_id`
- pricing plans and pricing rates remain upserted by stable ids
- repeated bootstrap applies the same linkage without duplicating dirty data

## Why this design

- operators need two answers at the same time:
  - what the upstream provider reference price is
  - what cost class and sell-side class the platform used for this request
- duplicating full provider-model price matrices into pricing plans would couple catalog and billing too tightly
- keeping `model-price` exact and `pricing-plan` class-based preserves:
  - high cohesion
  - lower coupling
  - easier price updates
  - safer additive overrides when pricing policy changes

## Verified

- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Next

- keep introducing new pricing posture classes through additive packs instead of editing the baseline pricing files in place
- if admin operators need more explicit comparison views, add read-model or UI summaries that join:
  - `provider-model`
  - `model-price`
  - `pricing-plan`
  - `pricing-rate`
  - request metering facts
