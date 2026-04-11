# 2026-04-10 Bootstrap Request Metering Billing Closure Step Update

## Summary

This step closes another commercial bootstrap integrity gap in the request-level evidence chain:

- `request_meter_facts` already validated account ownership, channel/model availability, and pricing-plan references
- `billing_events` already validated snapshot, provider/channel, and catalog/provider-model closure
- but there was still no explicit bootstrap contract tying a request metering record back to the billing evidence produced for the same gateway request

That allowed seeded request usage to remain structurally valid even if the billing evidence for the same request was missing or described a different provider execution story.

## What Changed

### 1. Added request-metering to billing existence closure

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- every `request_meter_facts.gateway_request_ref` must resolve to exactly one `billing_events.reference_id`

This makes the request evidence chain explicit:

- request metering records the canonical gateway request usage fact
- billing records the canonical commercial event for that same request
- bootstrap no longer accepts request usage that cannot be replayed into billing evidence

### 2. Added request-metering to billing execution-context consistency validation

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- resolved billing evidence must match the request metering record on:
  - `provider_code` vs `provider_id`
  - `channel_code` vs `channel_id`
  - `capability_code` vs `capability`
  - `actual_provider_cost` vs `upstream_cost` when metering has a provider-cost value

This prevents the same request from carrying two different upstream execution stories in bootstrap data.

### 3. Added regression tests for request-metering to billing drift

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Added regression tests for:

- request metering without billing evidence
- request metering whose provider/channel execution context drifts from billing evidence
- request metering whose capability drifts from billing evidence
- request metering whose provider cost drifts from billing evidence

### 4. Hardened bootstrap test fixtures for future closure work

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Test fixture support now injects a valid local billing event for the default request-metering fixture when a test overrides unrelated billing/snapshot data.

This keeps older bootstrap regression tests focused on the contract they intend to exercise instead of failing earlier on newly-added request-level closure rules.

## Data Audit Result

No `/data` fixes were required for this slice.

Repository data already satisfies the new contract:

- every seeded `request_meter_facts.gateway_request_ref` resolves to billing evidence
- the matched billing event already agrees on provider, channel, capability, and upstream cost

That means the stronger contract is now enforced in code instead of only being true by convention.

## Verification

Passed:

- `cargo test -p sdkwork-api-app-runtime request_meter_fact_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Result

Bootstrap data now enforces a stronger commercial replay chain:

- request metering cannot exist without matching billing evidence
- billing evidence cannot silently describe a different provider/channel/capability path for the same request
- upstream provider cost drift is rejected before bootstrap completes

This improves deployability for commercial audit trails, request replay diagnostics, and account-kernel reconciliation.
