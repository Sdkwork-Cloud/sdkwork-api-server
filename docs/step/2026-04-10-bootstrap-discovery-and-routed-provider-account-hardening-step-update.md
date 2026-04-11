# 2026-04-10 Bootstrap Discovery And Routed Provider Account Hardening Step Update

## Summary

This step closes two remaining bootstrap risks that were still weakening deployment readiness:

1. runtime bootstrap discovery could prefer stale nested `bin/data` over the repository root `data`
2. route config candidate providers could still be visible in shipped profiles even when they had no enabled executable `provider-account`

The result is a tighter commercial-readiness contract for both packaged startup and repository-backed dev/prod bootstrap.

## What Changed

### 1. Fixed bootstrap data root discovery priority

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/mod.rs`

New behavior:

- ancestor `data/` roots are still discovered automatically
- when both outer and nested roots exist on the same path chain, the outer root now wins
- this prevents `bin/data` from silently overriding the richer repository `data/` packs during local packaged-style startup
- if only nested `bin/data` exists, it remains usable

This preserves install-time flexibility without letting stale mirrors downgrade runtime bootstrap quality.

### 2. Hardened routed provider-account coverage

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- every routed provider referenced by `routing_profiles.providers` must have at least one enabled `provider-account`
- every routed provider referenced by `routing_policies.providers` must have at least one enabled `provider-account`
- every routed provider referenced by `project_preferences.providers` must have at least one enabled `provider-account`

The previous validation only covered `default_provider_id`. That still allowed shipped route candidate lists to advertise providers that could never actually execute.

Now the shipped route matrix must be executable across the full declared candidate set, not just the default path.

### 3. Upgraded the bootstrap test fixture to real multi-provider execution coverage

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Changed fixture behavior:

- the local bootstrap pack now seeds enabled default accounts for:
  - `provider-openrouter-main`
  - `provider-siliconflow-main`
  - `provider-ollama-local`

This keeps the default test pack aligned with the intended execution model:

- route config chooses providers
- provider-account makes those routed providers executable
- runtime binds through the account's `execution_instance_id`

### 4. Added regression coverage for both deployment and routing gaps

Added tests for:

- preferring outer repository `data` before nested `bin/data`
- rejecting route candidate providers without enabled provider-accounts
- preserving the stricter default-provider validation after the fixture was expanded

## Verification

Passed:

- `cargo test -p sdkwork-api-app-runtime bootstrap_push_candidate_roots_prefers_outer_repository_data_before_nested_bin_data -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_routing_candidates_without_enabled_account -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_routing_profile_default_provider_without_enabled_account -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_loads_bootstrap_profile_data_pack -- --nocapture`

## Result

Bootstrap behavior is now stricter and safer in two important ways:

- packaged or bin-launched runtime no longer prefers a weaker nested data mirror when a richer root data pack is available
- shipped route configs cannot declare non-executable provider candidates

That keeps `/data` closer to a true deployable source of truth instead of a loosely referential seed set.
