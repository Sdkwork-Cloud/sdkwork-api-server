# 2026-04-10 Enterprise Manual Contract Walkthrough Step Update

## Completed

### Step 1 - Added a dedicated additive enterprise walkthrough pack

- added [`2026-04-global-enterprise-manual-contract-walkthrough.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/updates/2026-04-global-enterprise-manual-contract-walkthrough.json)
- kept the change inside the existing bootstrap data-pack contract:
  - no importer changes
  - no baseline rewrite
  - no schema drift

### Step 2 - Added enterprise marketing scaffolding

- added [`2026-04-global-enterprise-manual-contract-walkthrough.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/marketing/2026-04-global-enterprise-manual-contract-walkthrough.json)
- seeded a contract-oriented enterprise funnel:
  - `enterprise-contract-credit-500`
  - `ENTERPRISE500`
  - `campaign-enterprise-contract-2026`
  - `budget-enterprise-contract-2026`
- kept the records aligned with the existing coupon-first commercial model:
  - stable template key
  - reusable shared-code campaign
  - explicit `enterprise_contract` target restriction

### Step 3 - Added pending manual-review commerce samples

- added [`2026-04-global-enterprise-manual-contract-walkthrough.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/commerce/2026-04-global-enterprise-manual-contract-walkthrough.json)
- seeded an install-ready enterprise contract order chain:
  - `order-global-enterprise-contract-2026`
  - `attempt-global-enterprise-contract-2026`
  - `payment-event-global-enterprise-contract-2026`
  - `recon-run-global-enterprise-contract-2026`
  - `recon-item-global-enterprise-contract-2026`
- the walkthrough intentionally stays in a pending manual-finance posture:
  - order status `pending_payment`
  - payment attempt status `pending_review`
  - reconciliation run status `running`
  - reconciliation item status `open`

### Step 4 - Added enterprise async operations context

- added [`2026-04-global-enterprise-manual-contract-walkthrough.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/jobs/2026-04-global-enterprise-manual-contract-walkthrough.json)
- seeded a realistic enterprise drafting job chain on the official Anthropic provider:
  - `job-global-enterprise-sow-draft`
  - attempt `9816`
  - asset `asset-global-enterprise-sow-draft-md`
  - callback `10816`
- this keeps the walkthrough commercially richer than a pure checkout sample by showing how pre-sales or contract operations can already use the seeded official channel/provider/model mapping

### Step 5 - Advanced reconciliation and wired both profiles

- added [`2026-04-global-enterprise-manual-contract-walkthrough.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/account-reconciliation/2026-04-global-enterprise-manual-contract-walkthrough.json)
- advanced the global default reconciliation checkpoint to the latest shipped commercial order:
  - `order-global-enterprise-contract-2026`
- updated:
  - [`prod.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/profiles/prod.json)
  - [`dev.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/profiles/dev.json)
- both environments now inherit the same enterprise manual-review storyline before any dev-only overlays are applied

## Why this design

- the existing canonical model remains correct:
  - `channel-model` = inventor catalog
  - `provider-model` = provider-supported subset
  - `model-price` = provider reference price
  - `pricing-plan` / `pricing-rate` = cost and retail posture
- the missing capability was not another schema layer; it was a richer install-ready enterprise narrative that exercises:
  - official provider routing
  - proxy-safe subset assumptions
  - operator-readable pending finance workflows
  - reconciliation advancement on later commercial orders
- packaging the scenario as one additive update pack preserves:
  - high cohesion
  - low coupling
  - idempotent replay
  - update-safe evolution through stable-key last-wins merge

## Verified

- `cargo test -p sdkwork-api-product-runtime product_runtime_bootstraps_repository_default_data_pack -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Next

- extend the same additive pattern for more enterprise postures instead of mutating baseline data:
  - invoice-paid enterprise renewals
  - regional distributor approval queues
  - multi-provider procurement walkthroughs
- keep admin-side provider model management explicit so proxy providers only expose the `provider-model` subset they actually serve
