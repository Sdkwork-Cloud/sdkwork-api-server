# 2026-04-10 Provider Proxy Account Kernel Step Update

## Completed

### Step 1 - Closed the remaining provider-operations finance-evidence gap

- `2026-04-global-provider-operations-readiness` already shipped install-ready operational samples for:
  - `group-official-openai-live`
  - `group-official-claude-live`
  - `group-official-gemini-live`
  - `group-proxy-openrouter-live`
  - `group-proxy-siliconflow-live`
  - `group-local-ollama-edge`
- those samples already had:
  - routing profiles
  - compiled routing snapshots
  - decision logs
  - billing events
- they still lacked matching account-kernel evidence:
  - request holds
  - request metering facts and metrics
  - request settlements
  - ledger captures

That left the provider-operations experience incomplete in admin and runtime: operators could inspect the route and billing surfaces, but not the same shipped flow through balance, hold, and settlement views.

### Step 2 - Added a dedicated provider-operations account-kernel pack

- added `data/updates/2026-04-global-provider-operations-account-kernel.json`
- wired it into both:
  - `data/profiles/prod.json`
  - `data/profiles/dev.json`
- added grouped JSON in:
  - `data/account-benefit-lots/2026-04-global-provider-operations-account-kernel.json`
  - `data/account-holds/2026-04-global-provider-operations-account-kernel.json`
  - `data/account-ledger/2026-04-global-provider-operations-account-kernel.json`
  - `data/request-metering/2026-04-global-provider-operations-account-kernel.json`
  - `data/request-settlements/2026-04-global-provider-operations-account-kernel.json`

The pack keeps high cohesion by covering one operational slice:

- official live provider groups
- main proxy provider groups
- local provider group

without mixing in unrelated dev-only tenant data or channel-specific proxy overlays.

### Step 3 - Closed the OpenRouter channel-proxy finance-evidence gap

- `2026-04-global-openrouter-channel-coverage` already shipped ready-to-use proxy-first route samples for:
  - xAI via OpenRouter
  - Moonshot via OpenRouter
  - Mistral via OpenRouter
- those route samples already had:
  - provider-model coverage
  - proxy price rows
  - route profiles
  - API key groups
  - observability samples
  - billing samples
- they still lacked account-kernel request evidence

That meant the bootstrap could demonstrate proxy-first route selection, but not end-to-end commercial tracing for those same proxy-first flows.

### Step 4 - Added a dedicated OpenRouter channel account-kernel pack

- added `data/updates/2026-04-global-openrouter-channel-account-kernel.json`
- wired it into both repository profiles
- added grouped JSON in:
  - `data/account-benefit-lots/2026-04-global-openrouter-channel-account-kernel.json`
  - `data/account-holds/2026-04-global-openrouter-channel-account-kernel.json`
  - `data/account-ledger/2026-04-global-openrouter-channel-account-kernel.json`
  - `data/request-metering/2026-04-global-openrouter-channel-account-kernel.json`
  - `data/request-settlements/2026-04-global-openrouter-channel-account-kernel.json`

This keeps update evolution modular:

- channel-specific proxy evidence lives with the channel-proxy pack
- provider-operations evidence lives with the provider-operations pack

so future catalog, pricing, or route refreshes can change one without rewriting the other.

### Step 5 - Canonical data contract preserved

- `channel` stayed canonical
- `provider` stayed executable official / proxy / local endpoint
- `provider-model` remained the source of truth for proxy subset support
- request metering continued to validate `(channel_code, model_code, provider_code)` against the shipped catalog
- settlements still carry:
  - provider cost
  - retail charge
- ledger entries still materialize only the account-side quantity and amount trail

No alternate initialization path or special-case importer logic was introduced. Everything still flows through the same profile-driven, additive, idempotent `/data` bootstrap pipeline.

### Step 6 - Repository baseline raised again

- app-runtime repository discovery coverage now proves that shipped provider-operations and channel-proxy samples are visible in:
  - holds
  - settlements
  - ledger history
  - summarized global account balance
- product-runtime bootstrap expectations were raised to the richer global seed baseline for both prod and dev

## Verified

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_discovers_repository_bootstrap_profile_data_pack -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

Both commands passed after the new provider/proxy account-kernel packs were added.

## Next

- continue pairing any default “ready-to-operate” proxy or local route pack with matching account-kernel evidence
- keep update packs narrowly scoped so billing, pricing, routing, and financial evidence can evolve independently without baseline rewrites
- if more default marketplace channels are added later, prefer one channel-scoped proxy account-kernel pack per coverage family rather than one oversized global rewrite
