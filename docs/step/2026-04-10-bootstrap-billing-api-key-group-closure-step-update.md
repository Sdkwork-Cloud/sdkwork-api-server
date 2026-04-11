# 2026-04-10 Bootstrap Billing API Key Group Closure Step Update

## Summary

This step closes another commercial bootstrap integrity gap around customer-key attribution:

- `billing_events.api_key_hash` already had to reference a known gateway API key hash
- `billing_events.api_key_group_id` already had to reference a valid API key group in the same workspace
- but bootstrap still did not require those two references to describe the same actual gateway key

That allowed seeded billing data to remain structurally valid while claiming:

- one concrete customer key hash
- but a different API key group identity

for the same billing event.

## What Changed

### 1. Added billing-to-gateway-key group/workspace closure

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- when `billing_events.api_key_hash` is present, the resolved `gateway_api_keys` record must match the billing event on:
  - `tenant_id`
  - `project_id`
  - `api_key_group_id`

This turns billing-side key attribution into an explicit bootstrap contract instead of a convention.

### 2. Added regression coverage for key-hash/group drift

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Added regression test for:

- a billing event whose `api_key_hash` resolves to a real gateway key, but whose `api_key_group_id` claims a different group

The test fixture is intentionally shaped so older workspace checks still pass; it only fails once the new hash-to-group closure is enforced.

## Data Audit Result

No `/data` changes were required for this slice.

Repository seed data already satisfied the stronger contract:

- every seeded billing event that carries an `api_key_hash`
- already matched the referenced gateway API key on workspace and API key group identity

This change therefore codifies an existing commercial truth rather than introducing a new data convention.

## Verification

Passed:

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_billing_api_key_hash_mismatched_gateway_key_group -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Result

Bootstrap data now enforces a stronger customer-key attribution chain:

- billing key hashes must resolve to the same workspace they bill against
- billing key hashes must resolve to the same API key group they claim
- API key group drift is rejected before bootstrap completes

This improves seeded deployability for billing attribution, workspace forensics, and customer-key auditability.
