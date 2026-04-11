# 2026-04-10 OpenRouter Channel Coverage Step Update

## Completed

### Step 1 - Closed the proxy-binding consistency gap

- identified that `provider-openrouter-main` already declared default channel bindings for:
  - `xai`
  - `moonshot`
  - `mistral`
- repository seed data previously lacked matching default:
  - `provider-model` rows
  - `model-price` rows
  - route profiles
  - operator-facing traffic groups
  - observability samples
  - billing samples

This meant the metadata implied broader default support than the shipped bootstrap data could actually demonstrate.

### Step 2 - Added a dedicated additive update pack

- added `data/updates/2026-04-global-openrouter-channel-coverage.json`
- wired it into both:
  - `data/profiles/prod.json`
  - `data/profiles/dev.json`

The pack extends existing seed data without rewriting baseline files, so repeated bootstrap remains idempotent and future pricing/model refreshes can continue through new additive packs.

### Step 3 - Expanded provider-model and pricing coverage

- added OpenRouter provider-model coverage for:
  - `xai / grok-4`
  - `moonshot / kimi-k2.5`
  - `mistral / mistral-large-latest`
- added matching proxy price rows for the same canonical channel/model combinations
- kept `price_source_kind = proxy` and explicit billing notes so operators can distinguish official vendor pricing from marketplace proxy reference pricing

### Step 4 - Added ready-to-use route configs

- added new default profiles:
  - `profile-global-openrouter-xai`
  - `profile-global-openrouter-moonshot`
  - `profile-global-openrouter-mistral`
- each profile is proxy-first and keeps the corresponding official provider as fallback
- added matching model-pattern policies for:
  - `grok-*`
  - `kimi-*`
  - `mistral-large-*`

This gives fresh installs more directly usable route options without forcing operators to assemble cross-vendor fallback chains by hand.

### Step 5 - Added operator-facing bootstrap evidence

- added dedicated API key groups for each new OpenRouter route posture
- added compiled routing snapshots and routing decision logs for:
  - xAI via OpenRouter
  - Moonshot via OpenRouter
  - Mistral via OpenRouter
- added billing sample events for the same flows

This improves dev/prod first-run experience in admin because the catalog, routing, and commercial surfaces now show realistic multi-vendor proxy examples instead of only generic OpenRouter traffic.

### Step 6 - Architectural rule clarified

- documented that shipped proxy/local `channel_binding` rows must be backed by:
  - active `provider-model` coverage
  - active `model-price` coverage
  - default operator-facing route data when the binding is presented as ready-to-use

That keeps bootstrap data high-cohesion, low-coupling, and update-safe: bindings are no longer allowed to drift into "declared but unroutable" territory in the repository seed set.

## Verified

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_discovers_repository_bootstrap_profile_data_pack -- --nocapture`

The command passed in the current workspace after the new update pack was added.

## Next

- continue expanding proxy-marketplace coverage with the same rule set for any newly declared bindings
- keep proxy price rows refreshable only through additive update packs
- consider adding account-kernel metering and settlement samples for the new OpenRouter channel-specific flows if commercial debugging needs more per-route financial evidence
