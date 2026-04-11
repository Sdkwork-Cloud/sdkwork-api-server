# 2026-04-10 Bootstrap Tenant Accessible Route Contract Step Update

## Summary

This step closes another route-executability gap in bootstrap data:

- a workspace-bound route could pass validation if a provider had an executable `provider-account`
- but that account could still belong to a different tenant

That was still "globally executable" but not actually reachable for the tenant that owned the route.

## What Changed

### 1. Added tenant-aware executable provider-account coverage for workspace-bound routes

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract for workspace-bound route data:

- `routing_profiles` now require every routed/default provider to have an executable provider-account accessible to `routing_profiles.tenant_id`
- `project_preferences` now require every routed/default provider to have an executable provider-account accessible to the owning tenant of `project_preferences.project_id`

Tenant accessibility follows the same owner-scope rule as runtime selection:

- non-tenant accounts are shared
- `owner_scope == "tenant"` accounts only count for their `owner_tenant_id`

This keeps bootstrap route config aligned with runtime provider-account selection semantics.

### 2. Preserved the broader global contract where tenant context does not exist

Global `routing_policies` still use the broader executable-account check because they are not bound to one tenant/project workspace in bootstrap data.

That avoids over-constraining global policy records while still hardening the workspace-bound route layer.

### 3. Added regression coverage for foreign-tenant account leakage

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Added regression tests for:

- routing profiles whose only executable provider-account belongs to another tenant
- project preferences whose only executable provider-account belongs to another tenant

## Verification

Passed:

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_routing_profile_provider_with_only_foreign_tenant_account -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_project_preferences_provider_with_only_foreign_tenant_account -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_loads_bootstrap_profile_data_pack -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_routing_candidates_without_enabled_account -- --nocapture`

## Result

Bootstrap routing validation now matches tenant-aware runtime behavior more closely:

- workspace-bound routes must be executable for that workspace's tenant
- foreign-tenant accounts no longer make a route look valid

This further reduces the chance of shipping `/data` packs that are structurally correct but operationally unreachable for real tenant traffic.
