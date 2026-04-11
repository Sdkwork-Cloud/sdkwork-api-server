# 2026-04-10 Asia Official Account Kernel Step Update

## Completed

### Step 1 - Closed the APAC official finance-evidence gap

- `2026-04-global-asia-official-operations` already made ERNIE and MiniMax visible in:
  - API key groups
  - compiled routing snapshots
  - routing decision logs
  - billing events
- production and dev bootstrap still stopped short of the account kernel:
  - no dedicated account benefit lot for APAC official traffic
  - no request holds for the shipped ERNIE / MiniMax official flows
  - no request metering facts or metrics
  - no request settlements
  - no ledger entries tying those official requests back to commercial balance state

That left the admin/runtime experience asymmetric: route and billing evidence existed, but the same shipped traffic could not be inspected end-to-end through the balance, hold, and settlement surfaces.

### Step 2 - Added a dedicated additive update pack

- added `data/updates/2026-04-global-asia-account-kernel-operations.json`
- wired it into both:
  - `data/profiles/prod.json`
  - `data/profiles/dev.json`
- declared dependencies on:
  - `2026-04-global-asia-official-operations`
  - `2026-04-account-kernel-commercial-foundation`

This keeps the bootstrap evolution additive and idempotent. The pack extends the existing global commercial account instead of rewriting baseline files in place.

### Step 3 - Extended `/data` across cohesive account-kernel domains

- added a dedicated APAC official reserve lot in:
  - `data/account-benefit-lots/2026-04-global-asia-account-kernel-operations.json`
- added ERNIE and MiniMax request holds plus hold allocations in:
  - `data/account-holds/2026-04-global-asia-account-kernel-operations.json`
- added request metering facts and metrics in:
  - `data/request-metering/2026-04-global-asia-account-kernel-operations.json`
- added request settlements in:
  - `data/request-settlements/2026-04-global-asia-account-kernel-operations.json`
- added matching grant/capture ledger entries and allocations in:
  - `data/account-ledger/2026-04-global-asia-account-kernel-operations.json`

Each record set reuses stable IDs and preserves the existing ownership chain:

- tenant `1001`
- organization `2001`
- account `7001001`
- user `3001`
- official ERNIE request `610002`
- official MiniMax request `610003`

### Step 4 - Preserved the pricing and routing relationship model

- request metering points to canonical:
  - `channel_code`
  - `model_code`
  - `provider_code`
- settlements carry both:
  - provider cost
  - retail charge
- ledger captures consume credit quantity while preserving monetary amount visibility
- the account kernel pack does not introduce a second pricing system; it materializes operational evidence on top of the existing:
  - `model-price`
  - `pricing-plan`
  - `pricing-rate`
  - route config

This keeps official pricing, provider pricing, and internal account charging explainable instead of collapsing them into one opaque record.

### Step 5 - Strengthened repository regression coverage

- expanded the app-runtime repository discovery test so production bootstrap now proves the presence of:
  - APAC official holds
  - settlements
  - ledger captures
  - updated global account balance
- updated product-runtime bootstrap expectations to treat the richer APAC account-kernel data as part of the default repository baseline rather than as an accidental count change

## Verified

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_discovers_repository_bootstrap_profile_data_pack -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

All commands passed in the current workspace after the new APAC official account-kernel pack was added.

## Next

- keep future official route-matrix additions paired with account-kernel evidence when the bootstrap profile presents them as install-ready operational examples
- continue preferring additive `/data/updates/*.json` packs for regional catalog, pricing, and financial evidence refreshes
- if admin surfaces expose per-channel commercial diagnostics, use these same request / hold / settlement IDs as the default inspectable walkthrough set
