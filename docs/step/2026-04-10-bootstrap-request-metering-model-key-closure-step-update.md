# 2026-04-10 Bootstrap Request Metering Model Key Closure Step Update

## Summary

This step tightens the request-level bootstrap evidence chain again, this time around routed model identity and customer key identity:

- the previous slice already required `request_meter_facts.gateway_request_ref` to resolve to billing evidence
- and it already required provider, channel, capability, and upstream cost to agree
- but it still allowed the same request to carry a different canonical routed model or a different API key hash between metering and billing

That left a gap in commercial replay semantics:

- request metering could describe one canonical model while billing described another
- request metering could point at one customer key while billing pointed at another

## What Changed

### 1. Added request-metering to billing model closure

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- every request metering record that resolves to a billing event must satisfy:
  - `request_meter_facts.model_code == billing_events.route_key`

This formalizes the intended meaning of the metering-side model code:

- `model_code` is the canonical routed model
- `usage_model` can still differ on the billing side for provider-specific execution mappings
- but the routed model identity itself can no longer drift between metering and billing

### 2. Added request-metering to billing API key hash closure

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- every request metering record that resolves to a billing event must satisfy:
  - `request_meter_facts.api_key_hash == billing_events.api_key_hash`

This makes the customer-key identity chain explicit for the same gateway request.

### 3. Added regression tests for model and key drift

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Added regression tests for:

- metering `model_code` drifting from billing `route_key`
- metering `api_key_hash` drifting from billing `api_key_hash`

### 4. Normalized the local bootstrap test fixture

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

The default test bootstrap pack now uses a consistent routed-model story for the local demo request:

- request metering `model_code`
- billing `route_key`
- routing snapshot `route_key`

all align on the same canonical model.

This keeps future bootstrap integrity work from being blocked by an old fixture inconsistency.

## Data Audit Result

No `/data` changes were required for this slice.

Repository seed data already satisfied the stronger contract:

- metering `model_code` already matched billing `route_key`
- metering `api_key_hash` already matched billing `api_key_hash`

The change therefore hardens an existing commercial truth instead of forcing the repository data into a new convention.

## Verification

Passed:

- `cargo test -p sdkwork-api-app-runtime request_meter_fact_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Result

Bootstrap data now enforces a stronger commercial request identity chain:

- request metering and billing must agree on the canonical routed model
- request metering and billing must agree on the customer API key hash
- routed model drift is rejected before bootstrap completes
- customer-key identity drift is rejected before bootstrap completes

This improves seeded deployability for commercial replay, auditability, customer-key attribution, and request evidence consistency.
