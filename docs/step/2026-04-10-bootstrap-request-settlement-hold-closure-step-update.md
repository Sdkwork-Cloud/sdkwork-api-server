# 2026-04-10 Bootstrap Request Settlement Hold Closure Step Update

## Summary

This step closes another commercial bootstrap integrity gap inside the account-kernel evidence chain:

- previous slices already required request settlements to reference a valid request meter fact
- and they already required settlement ownership, estimated hold, captured amount, and provider cost to agree with request metering
- but bootstrap still did not require a settlement to agree with the linked account hold on hold-side quantities

That allowed seeded settlement records to remain structurally valid while describing a different hold-release story than the hold actually recorded.

## What Changed

### 1. Added settlement-to-hold quantity closure

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- when `request_settlements.hold_id` is present, the resolved hold must match the settlement on:
  - `estimated_credit_hold` vs `estimated_quantity`
  - `captured_credit_amount` vs `captured_quantity`
  - `released_credit_amount` vs `released_quantity`

This turns hold-side quantity replay into an explicit bootstrap contract.

### 2. Added regression coverage for settlement-to-hold drift

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Added regression tests for:

- a settlement whose `released_credit_amount` drifts from the linked hold
- a settlement whose `estimated_credit_hold` drifts from the linked hold

These tests are intentionally shaped so older settlement-to-request-metering checks still pass; they only fail once the new hold-side closure is enforced.

## Data Audit Result

No `/data` changes were required for this slice.

Repository seed data already satisfied the stronger contract:

- every settlement linked to a hold already matched the hold on estimated, captured, and released quantities

That means the stronger rule codifies an existing commercial truth instead of introducing a new bootstrap convention.

## Verification

Passed:

- `cargo test -p sdkwork-api-app-runtime request_settlement_released_amount_mismatched_hold -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime request_settlement_estimated_hold_mismatched_hold -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime request_settlement_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Result

Bootstrap data now enforces a stronger account-kernel settlement contract:

- settlements cannot claim a different release quantity than the hold actually records
- settlements cannot claim a different estimated hold quantity than the hold actually records
- settlement and hold evidence remain replayable as one coherent account-kernel story

This improves deployability for settlement forensics, credit-release auditability, and commercial seed-data trustworthiness.
