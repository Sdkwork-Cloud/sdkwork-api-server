# 2026-04-10 Pricing Posture Operations Linkage Step Update

## Completed

### Step 1 - Added a dedicated operations linkage update pack

- added [`2026-04-global-pricing-posture-operations-linkage.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/updates/2026-04-global-pricing-posture-operations-linkage.json)
- kept the extension additive and dependency-driven:
  - depends on the commerce walkthrough pack
  - depends on provider-operations readiness
  - depends on account-kernel commercial foundation

### Step 2 - Extended pricing posture walkthroughs into billing telemetry

- added [`2026-04-global-pricing-posture-operations-linkage.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/billing/2026-04-global-pricing-posture-operations-linkage.json)
- added new billing samples aligned to the newly seeded commercial walkthrough orders:
  - `billing-prod-openai-official-direct-2026`
  - `billing-prod-ollama-local-edge-2026`
- each event binds back to an existing route surface:
  - official direct -> `group-official-openai-live` / `snapshot-prod-openai-official-live`
  - local edge -> `group-local-ollama-edge` / `snapshot-prod-ollama-local`

### Step 3 - Added richer async job walkthroughs

- added [`2026-04-global-pricing-posture-operations-linkage.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/jobs/2026-04-global-pricing-posture-operations-linkage.json)
- added new async job chains for operations teams:
  - `job-global-official-direct-capacity-plan`
  - `job-global-edge-local-capacity-audit`
- each chain includes:
  - job
  - attempt
  - asset
  - callback

### Step 4 - Advanced account reconciliation state

- added [`2026-04-global-pricing-posture-operations-linkage.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/account-reconciliation/2026-04-global-pricing-posture-operations-linkage.json)
- advanced the stable `(account_id, project_id)` reconciliation checkpoint for the global default commercial account:
  - previous latest order -> `order-global-growth-bootstrap`
  - current latest walkthrough order -> `order-edge-local-ollama-2026`
- this keeps reconciliation state aligned with the newest shipped commerce sample instead of leaving the portal/admin ledger summary behind the latest bootstrap order

### Step 5 - Wired prod and dev profiles to the new operations slice

- updated:
  - [`prod.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/profiles/prod.json)
  - [`dev.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/profiles/dev.json)
- both environments now inherit the same richer operational storyline on top of the pricing posture commerce walkthrough

## Why this design

- the previous iteration completed the commercial checkout story, but not the operational one
- install-ready data needs the full chain:
  - route snapshot
  - billing event
  - async follow-up job
  - reconciliation checkpoint
- keeping these as a dependent additive pack preserves:
  - low coupling between commerce and runtime governance slices
  - safe incremental evolution
  - idempotent replay through stable IDs and last-wins reconciliation override

## Verified

- `cargo test -p sdkwork-api-product-runtime product_runtime_bootstraps_repository_default_data_pack -- --nocapture`

## Next

- extend the same pattern to more regional or enterprise walkthrough packs instead of growing baseline files
- if portal/admin surfaces need even more ready-to-demo depth, add read-only examples for:
  - manual settlement review queues
  - provider issue escalation timelines
  - cost-versus-retail variance summaries joined from billing and pricing posture data
