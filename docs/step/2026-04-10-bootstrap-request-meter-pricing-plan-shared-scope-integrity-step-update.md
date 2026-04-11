# 2026-04-10 Bootstrap Request Meter Pricing Plan Shared Scope Integrity Step Update

## What Changed

- Hardened bootstrap validation for:
  - `request_meter_facts.cost_pricing_plan_id`
  - `request_meter_facts.retail_pricing_plan_id`
- Added explicit `ownership_scope` to `PricingPlanRecord`:
  - `workspace`
  - `platform_shared`
- A request meter fact now passes bootstrap only when its referenced pricing plan is:
  - owned by the same `tenant_id + organization_id`
  - or explicitly marked `platform_shared`

## Why This Matters

- Previous validation only proved that the referenced pricing plan:
  - existed
  - was active when used by request metering
- That was not sufficient for a commercial billing system.
- Without an ownership rule, request usage could silently attach to another workspace's pricing posture and still pass structural validation.
- At the same time, the repository already contains intentional shared pricing posture classes used across dev sample traffic.
- The correct rule is therefore not "same workspace only".
- The correct rule is "same workspace unless the plan is explicitly modeled as a platform shared pricing class".

## Repository Audit Outcome

- Re-audited the real repository data across:
  - `data/pricing/*.json`
  - `data/request-metering/*.json`
  - additive `data/updates/*.json`
- Confirmed the `dev` cross-workspace request-meter references are intentional and come from pricing-governance linkage packs, not accidental dirty data.
- The legitimate shared posture classes are the global pricing governance plans used by:
  - official direct traffic
  - marketplace proxy traffic
  - local edge traffic

## Data Model Impact

- `PricingPlanRecord` now carries explicit ownership semantics instead of inferring them from workspace ids alone.
- Seed data updates:
  - [`2026-04-global-provider-operations-readiness.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/pricing/2026-04-global-provider-operations-readiness.json)
  - [`2026-04-global-pricing-governance-linkage.json`](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/pricing/2026-04-global-pricing-governance-linkage.json)
- Shared pricing posture plans in those files are now marked with:
  - `"ownership_scope": "platform_shared"`
- Default behavior remains safe:
  - omitted `ownership_scope` deserializes to `workspace`
  - repeated bootstrap stays idempotent because pricing plans remain upserted by stable ids

## Storage Impact

- SQLite and Postgres pricing-plan persistence now both store and decode `ownership_scope`.
- SQLite migration adds `ai_pricing_plan.ownership_scope` with default `workspace`.
- Postgres migration adds `ai_pricing_plan.ownership_scope` with default `workspace`.
- This keeps old rows backward-compatible while allowing additive governance hardening.

## Test Coverage Added

- bootstrap rejects request meter facts that point to a cross-workspace pricing plan without shared scope
- bootstrap allows request meter facts that point to a `platform_shared` cross-workspace pricing plan
- sqlite account-kernel roundtrip now exercises persistence of `platform_shared` pricing plans
- postgres pricing surface regression test now validates the split-module storage implementation and `ownership_scope` handling

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_request_meter_fact_with_non_shared_cross_workspace_pricing_plan -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_allows_bootstrap_request_meter_fact_with_platform_shared_cross_workspace_pricing_plan -- --nocapture`
- `cargo test -p sdkwork-api-storage-sqlite sqlite_store_round_trips_canonical_account_kernel_records -- --nocapture`
- `cargo test -p sdkwork-api-storage-postgres --test account_kernel_pricing_surface -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- Any future global or shared pricing posture must declare `ownership_scope = platform_shared` in seed data instead of relying on cross-workspace ids alone.
- If additional shared scopes appear later, evolve `ownership_scope` as an explicit taxonomy rather than weakening request-meter lineage validation.
- Admin read models can later expose this field directly so operators can distinguish:
  - workspace-local pricing plans
  - reusable platform posture classes
