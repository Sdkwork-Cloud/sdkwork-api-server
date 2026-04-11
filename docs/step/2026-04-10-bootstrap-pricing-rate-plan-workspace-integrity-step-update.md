# 2026-04-10 Bootstrap Pricing Rate Plan Workspace Integrity Step Update

## What Changed

- Hardened bootstrap validation for `pricing_rates.pricing_plan_id`.
- A pricing rate now fails bootstrap unless its parent pricing plan belongs to the same `tenant_id + organization_id`.
- This makes pricing ownership explicit instead of relying only on id existence.

## Why This Matters

- Before this step, bootstrap validated that:
  - the parent pricing plan existed
  - active pricing rates only pointed at active pricing plans
  - provider/model/capability declarations were structurally valid
- But it did **not** verify that the pricing rate and the parent plan belonged to the same workspace ownership.
- That left a governance gap where a rate could reuse another tenant or organization's plan id and still pass structural validation.
- In a commercial billing system, that is a dangerous form of cross-workspace leakage.

## Scope Boundary

- This step validates only:
  - `pricing_rates.tenant_id == pricing_plans.tenant_id`
  - `pricing_rates.organization_id == pricing_plans.organization_id`
- It does **not** add a new rule for `request_meter_facts.cost_pricing_plan_id` or `retail_pricing_plan_id`.
- That broader lineage rule was audited separately and is not yet safe to enforce because current `dev` repository data intentionally reuses active pricing plans across local sample traffic.

## Repository Audit

- Re-audited merged `prod` and `dev` profile packs across `pricing/*.json`, `request-metering/*.json`, and all declared `updates/*.json`.
- Audit result for `prod`:
  - `PRICING_RATE_PLAN_WORKSPACE_MISMATCH=0`
  - `FACT_COST_PLAN_WORKSPACE_MISMATCH=0`
  - `FACT_RETAIL_PLAN_WORKSPACE_MISMATCH=0`
- Audit result for `dev`:
  - `PRICING_RATE_PLAN_WORKSPACE_MISMATCH=0`
  - `FACT_COST_PLAN_WORKSPACE_MISMATCH=3`
  - `FACT_RETAIL_PLAN_WORKSPACE_MISMATCH=3`

## Data Impact

- No repository `/data` seed files required changes for this rate-plan invariant.
- Existing `prod` and `dev` bootstrap packs already satisfy the new pricing rate ownership rule.
- The audit also documented why request-meter-plan ownership tightening was deferred instead of being mixed into this step.

## Test Coverage Added

- pricing rate rejects a parent pricing plan with mismatched tenant/organization ownership

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_pricing_rate_with_mismatched_parent_plan_workspace -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`
- Real `/data` audit:
  - `PROD PRICING_RATE_PLAN_WORKSPACE_MISMATCH=0 FACT_COST_PLAN_WORKSPACE_MISMATCH=0 FACT_RETAIL_PLAN_WORKSPACE_MISMATCH=0`
  - `DEV PRICING_RATE_PLAN_WORKSPACE_MISMATCH=0 FACT_COST_PLAN_WORKSPACE_MISMATCH=3 FACT_RETAIL_PLAN_WORKSPACE_MISMATCH=3`

## Follow-Up

- If pricing lineage tightening continues, the next step should likely be one of:
  - normalize the `dev` sample request-meter facts so plan ownership matches local workspace ownership, then enforce fact-to-plan ownership
  - or explicitly model shared/global pricing-plan reuse so bootstrap can distinguish intentional global plans from accidental cross-workspace leakage
