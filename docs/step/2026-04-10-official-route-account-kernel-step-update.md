# 2026-04-10 Official Route Account Kernel Step Update

## Completed

### Step 1 - Closed the official route matrix finance-evidence gap

- `2026-04-global-official-route-matrix` already made these official channels install-ready in catalog and routing:
  - DeepSeek
  - Qwen
  - Doubao
  - xAI
  - Moonshot
  - Zhipu
  - Hunyuan
  - Mistral
  - Cohere
- repository bootstrap already shipped for each of them:
  - official routing profile
  - live API key group
  - compiled routing snapshot
  - routing decision sample
  - billing event sample
- but those routes still had no matching account-kernel evidence:
  - no request hold
  - no request metering fact or metrics
  - no request settlement
  - no ledger capture trail

That meant the system could show operators that these routes existed, but could not walk the same default request through balance, hold, and settlement views.

### Step 2 - Added an additive account-kernel update pack

- added `data/updates/2026-04-global-official-route-account-kernel.json`
- wired it into both:
  - `data/profiles/prod.json`
  - `data/profiles/dev.json`
- kept the pack additive and update-safe by depending on:
  - `2026-04-global-official-route-matrix`
  - `2026-04-account-kernel-commercial-foundation`

This preserves the repository rule that richer startup data evolves through ordered update packs rather than baseline rewrites.

### Step 3 - Extended `/data` with cohesive official-route financial evidence

- added a dedicated reserve lot in:
  - `data/account-benefit-lots/2026-04-global-official-route-account-kernel.json`
- added request holds and hold allocations for the official route matrix in:
  - `data/account-holds/2026-04-global-official-route-account-kernel.json`
- added request metering facts and metrics in:
  - `data/request-metering/2026-04-global-official-route-account-kernel.json`
- added request settlements in:
  - `data/request-settlements/2026-04-global-official-route-account-kernel.json`
- added matching grant and settlement-capture ledger entries in:
  - `data/account-ledger/2026-04-global-official-route-account-kernel.json`

The new records cover request-level official traffic for:

- `deepseek-chat`
- `qwen-max`
- `doubao-seed-1.6`
- `grok-4`
- `kimi-k2.5`
- `glm-5`
- `hunyuan-t1-latest`
- `mistral-large-latest`
- `command-a-03-2025`

### Step 4 - Preserved the canonical channel/provider/model/pricing contract

- every request meter fact still resolves through canonical:
  - `channel_code`
  - `model_code`
  - `provider_code`
- every settlement still carries both:
  - provider cost
  - retail charge
- every ledger capture still consumes account credit quantity while preserving monetary amount visibility
- no new pricing table was introduced; this pack stays layered on top of:
  - `model-price`
  - `pricing-plan`
  - `pricing-rate`
  - route config

This keeps the default seed set high-cohesion and low-coupling: routing, pricing, and billing semantics remain canonical, while account-kernel records provide operational evidence instead of redefining pricing behavior.

### Step 5 - Regression coverage raised to the richer default baseline

- app-runtime repository discovery coverage now asserts that shipped official route matrix channels are visible in:
  - account holds
  - settlements
  - ledger history
  - summarized global balance
- product-runtime bootstrap expectations were raised so the richer global account-kernel seed set is treated as the new install-time baseline in both prod and dev profiles

## Verified

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_discovers_repository_bootstrap_profile_data_pack -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

Both commands passed after the new official route account-kernel pack was added.

## Next

- continue pairing any future “ready-to-use” official or proxy route pack with matching account-kernel evidence when the default experience is expected to be commercially inspectable
- extend the same pattern to proxy-marketplace route packs where admin should be able to inspect full cost-to-settlement traces, not just billing events
- keep evolving all of this through additive `/data/updates/*.json` manifests so profile-driven startup remains idempotent and update-safe
