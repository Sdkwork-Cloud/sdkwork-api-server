# 2026-04-10 Bootstrap Provider Executability Closure Step Update

## What Changed

- Hardened bootstrap validation so `billing_events.provider_id` must resolve to a tenant-accessible executable provider account.
- Hardened bootstrap validation so `request_meter_facts.provider_code` must resolve to at least one executable provider account in the current deployment.
- Added regression tests for:
  - billing evidence that only has a foreign-tenant-scoped provider account
  - request metering facts that reference a provider with no executable account

## Why This Matters

- Structural catalog linkage was already enforced, but bootstrap could still accept evidence records that pointed at providers the deployment could not actually execute.
- Commercial bootstrap data should reject "catalog-valid but operationally dead" providers before the environment starts serving traffic.
- Billing evidence uses string tenant scope, so it can safely enforce tenant-accessible executability.
- Request metering facts use numeric tenant/account ownership and do not have a stable string-tenant mapping in the bootstrap layer, so the safe invariant here is global executability rather than tenant-scoped executability.

## Data Impact

- No `/data` fixes were required.
- Current repository bootstrap data passed the stricter validation as-is, which confirms the seeded provider-account coverage is already commercially executable for the shipped dev/prod bootstrap packs.

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_billing_provider_with_only_foreign_tenant_account -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_request_meter_fact_without_executable_provider_account -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime request_meter_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`

## Follow-Up

- If bootstrap later gains a stable mapping between numeric request-meter tenants and string workspace tenants, `request_meter_facts.provider_code` can be tightened from global executability to tenant-accessible executability as well.
