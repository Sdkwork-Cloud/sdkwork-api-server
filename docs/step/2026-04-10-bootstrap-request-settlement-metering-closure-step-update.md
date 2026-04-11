# 2026-04-10 Bootstrap Request Settlement Metering Closure Step Update

## Summary

This step closes another commercial bootstrap integrity gap in the account-kernel chain:

- `request_settlements` already referenced accounts and optional holds
- but they did not need to resolve back to a canonical `request_meter_fact`
- and they did not need to match the metering-side accounting values for that request

That allowed seeded settlement data to look structurally valid while drifting away from the request usage record it was supposed to settle.

## What Changed

### 1. Added settlement-to-metering existence validation

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- every `request_settlements.request_id` must resolve to a seeded `request_meter_fact`

This makes the commercial chain explicit:

- request metering is the canonical usage fact
- request settlement is the canonical account-kernel settlement derived from that request

### 2. Added settlement-to-metering ownership and accounting consistency validation

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- settlement and metering must agree on:
  - tenant
  - organization
  - user
  - account
- settlement and metering must also agree on:
  - `estimated_credit_hold`
  - `captured_credit_amount` vs `request_meter_facts.actual_credit_charge`
  - `provider_cost_amount` vs `request_meter_facts.actual_provider_cost`

This prevents the same request from carrying two different commercial stories in bootstrap data.

### 3. Added regression tests for settlement/metering drift

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Added regression tests for:

- request settlements that reference no request metering fact
- request settlements whose captured amount drifts from the request metering fact

### 4. Repaired real `/data` pricing-governance drift

Updated:

- `data/request-metering/2026-04-global-pricing-governance-linkage.json`

Repository bootstrap verification exposed a real conflict:

- `request_settlements` and `account_holds` for APAC official requests `610002` and `610003` already agreed with billing evidence
- but `2026-04-global-pricing-governance-linkage.json` overrode the same `request_meter_fact` records with different `estimated_credit_hold` and `actual_provider_cost` values

Fix applied:

- aligned request metering for `610002` and `610003` back to the settled account-kernel and billing evidence
- preserved the pricing-plan linkage intent without allowing accounting drift

## Verification

Passed:

- `cargo test -p sdkwork-api-app-runtime request_settlement_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Result

Bootstrap data now enforces a stronger commercial accounting chain:

- request settlements cannot exist without a request metering fact
- settlement amounts cannot silently drift away from request metering
- pricing-governance update packs can no longer override metering facts in ways that contradict holds, settlements, and billing evidence

This improves seeded deployability for commercial diagnostics, account history inspection, and financial replay.
