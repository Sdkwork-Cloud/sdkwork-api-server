# 2026-04-10 Provider Account Routing First Slice Step Update

## Completed

### Step 1 - Closed the provider-account bootstrap gap in the repository seed set

- added `data/provider-accounts/default.json`
- seeded one default executable account for every shipped default provider:
  - official channels:
    - OpenAI
    - Anthropic
    - Gemini
    - DeepSeek
    - Qwen
    - Doubao
    - xAI
    - Moonshot
    - Zhipu
    - Hunyuan
    - Mistral
    - Cohere
  - proxy and local channels:
    - OpenRouter
    - SiliconFlow
    - Ollama
- bound every seeded account to the existing canonical `extension_instance.instance_id`
- added route-facing hints to each account:
  - `region`
  - `priority`
  - `weight`
  - `routing_tags`
  - `health_score_hint`
  - `latency_ms_hint`
  - `cost_hint`
  - `success_rate_hint`

This makes the repository bootstrap actually ship first-class provider-account data instead of only shipping provider records and leaving account routing empty at install time.

### Step 2 - Wired provider accounts into both shipped bootstrap profiles

- updated `data/profiles/dev.json`
- updated `data/profiles/prod.json`
- both profiles now include:
  - `provider_accounts: ["provider-accounts/default.json"]`

This keeps bootstrap profile behavior symmetric between dev and prod:

- same data shape
- same idempotent import path
- same catalog/runtime bridge available immediately after startup

### Step 3 - Implemented account-aware gateway planned execution

- extended planned execution runtime context with:
  - `provider_account_id`
  - `execution_instance_id`
- implemented account-aware selection in gateway resolution for planned execution flows
- selection now:
  - filters disabled accounts
  - skips accounts whose bound runtime instance is disabled or unloadable
  - respects tenant ownership when `owner_scope == "tenant"`
  - prefers exact requested-region match
  - then sorts by higher `priority`
  - then higher `weight`
  - then stable lexical `provider_account_id`
- selected execution now binds through the account's `execution_instance_id`
- effective upstream base URL now resolves in this order:
  - `provider_account.base_url_override`
  - `extension_instance.base_url`
  - `provider.base_url`

### Step 4 - Bound account execution to the right credential source

- planned execution now resolves account credentials from the selected runtime instance's `credential_ref`
- when an account-specific credential reference exists:
  - the gateway resolves that exact `(tenant_id, provider_id, key_reference)` credential
- when no account-specific credential reference exists:
  - the gateway falls back to provider-level secret resolution
- when an account-specific credential is absent:
  - the gateway falls back to the enabled official provider secret

This preserves the intended layered model:

- `channel -> model -> provider -> provider-account -> execution-instance`

and stops the gateway from accidentally pairing one account's runtime binding with another credential.

### Step 5 - Normalized admin UI routing strategy values to canonical backend enums

- updated `apps/sdkwork-router-admin/packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutingProfilesDialog.tsx`
- removed legacy UI defaults and option values:
  - `priority`
  - `balanced`
  - `latency_optimized`
  - `cost_optimized`
- the dialog now emits canonical backend values only:
  - `deterministic_priority`
  - `weighted_random`
  - `slo_aware`
  - `geo_affinity`

This keeps admin bootstrap, backend validation, and route-selection semantics aligned.

### Step 6 - Closed the runtime gap between planned account selection and actual upstream execution

- extended gateway runtime execution with descriptor-driven execution helpers so real relay calls can consume:
  - `provider_account_id`
  - `execution_instance_id`
  - account-specific `base_url`
  - account-specific resolved credential
- fixed the actual relay path so it no longer recomputes a provider-default execution binding after planned selection already chose a concrete provider-account
- wired descriptor execution into:
  - chat direct relay
  - chat planned relay
  - responses direct relay
  - responses stream relay
  - responses input-token relay
  - planned responses input-token relay
- preserved the two-stage routing contract:
  - stage 1: select provider by route policy
  - stage 2: select executable provider-account under that provider
  - execution then uses the selected account binding end-to-end

This closes the most important correctness hole in the first slice: previously the gateway could select the right provider-account in planning, but still execute against the provider's default runtime account at relay time.

## Verified

- `cargo test -p sdkwork-api-app-gateway --test provider_accounts -- --nocapture`
- `node --test apps/sdkwork-router-admin/tests/admin-routing-strategy-normalization.test.mjs`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_loads_bootstrap_profile_data_pack -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_applies_bootstrap_profile_data_idempotently -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

All commands passed after the provider-account seed pack, gateway account selection, and UI normalization changes landed.

## Next

- extend the same account-aware descriptor path into the remaining direct relay surfaces that still resolve provider secrets at provider granularity:
  - responses retrieve/cancel/delete/input-items/compact
  - music/video and other model-bound direct relay surfaces
- add additive `/data/updates/*.json` packs for richer multi-account defaults:
  - region-split official accounts
  - provider proxy sub-accounts
  - tenant-scoped managed accounts
- expose provider-account coverage more explicitly in admin:
  - per-provider model coverage
  - account-to-model eligibility
  - account-level pricing and SLO hints
