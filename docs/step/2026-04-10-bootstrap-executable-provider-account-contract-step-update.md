# 2026-04-10 Bootstrap Executable Provider Account Contract Step Update

## Summary

This step tightens the bootstrap data contract from "provider-account exists" to "provider-account is actually executable".

That closes a subtle but important commercial-readiness gap: a shipped route could still look executable if it had an enabled `provider-account`, even when that account was bound to a disabled runtime instance or a disabled installation.

## What Changed

### 1. Promoted provider-account into an executable bootstrap contract

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- if `provider_account.enabled == true`, its bound `execution_instance_id` must resolve to an enabled `extension_instance`
- if `provider_account.enabled == true`, that instance's `installation_id` must resolve to an enabled `extension_installation`

This means bootstrap data can no longer ship an "enabled" account that the runtime would immediately skip.

### 2. Strengthened route executability against executable accounts

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

Route validation now derives provider executability from accounts that are:

- enabled
- bound to an enabled instance
- bound through an enabled installation

So `routing_profiles`, `routing_policies`, and `project_preferences` now rely on executable provider-account coverage instead of a weaker "enabled row exists" signal.

### 3. Added regression coverage for fake-available account states

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Added regression tests for:

- enabled provider-account bound to a disabled extension instance
- enabled provider-account bound to a disabled installation

The local bootstrap fixture continues to seed executable accounts for:

- `provider-openrouter-main`
- `provider-siliconflow-main`
- `provider-ollama-local`

so the default development seed remains realistic for runtime planning and startup validation.

## Verification

Passed:

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_enabled_provider_account_with_disabled_extension_instance -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_enabled_provider_account_with_disabled_installation -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_routing_candidates_without_enabled_account -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_loads_bootstrap_profile_data_pack -- --nocapture`

## Result

Bootstrap data now enforces a stronger execution truth:

- enabled account means executable account
- executable route candidate means runtime-reachable provider

That reduces the risk of shipping catalog and route data that looks commercial-ready in admin but is not actually runnable after deployment.
