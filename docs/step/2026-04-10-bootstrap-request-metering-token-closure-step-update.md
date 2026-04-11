# 2026-04-10 Bootstrap Request Metering Token Closure Step Update

## Summary

This step tightens the commercial request evidence chain around token usage quantities:

- previous slices already required request metering to resolve to exactly one billing event
- and they already required provider, channel, capability, route key, API key hash, provider cost, timing window, and capture-state accounting to agree
- but bootstrap still did not require request-level token usage metrics to match the billing-side token counts for the same request

That left a gap where a seeded request could claim one token usage story in metering and a different token usage story in billing.

## What Changed

### 1. Added request-metering to billing token quantity closure

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- when a request resolves to billing evidence:
  - `billing_events.input_tokens > 0` requires at least one `request_meter_metrics.metric_code = token.input`
  - aggregated `token.input` quantity must equal `billing_events.input_tokens`
  - `billing_events.output_tokens > 0` requires at least one `request_meter_metrics.metric_code = token.output`
  - aggregated `token.output` quantity must equal `billing_events.output_tokens`

This converts token accounting from a convention into a bootstrap contract.

### 2. Added regression tests for missing and drifted token metrics

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Added regression tests for:

- billing input tokens without corresponding `token.input` metrics
- `token.output` metric quantity drifting from the matched billing event

The older request-meter-metric fixture tests were also normalized so they continue failing on their intended ownership/stage contracts instead of failing early on the new token closure.

## Data Audit Result

No `/data` changes were required for this slice.

Repository seed data already satisfied the stronger token contract:

- merged `prod` and `dev` profiles have no requests with billing token counts but missing token metrics
- merged `prod` and `dev` profiles already have matching `token.input` and `token.output` aggregates for every request with billing evidence

That means the stronger rule codifies an existing commercial truth instead of forcing the repository data into a new convention.

## Verification

Passed:

- `cargo test -p sdkwork-api-app-runtime request_meter_fact_without_token_input_metric_for_billing_tokens -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime request_meter_fact_token_output_mismatched_billing_event -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime request_meter_fact_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime request_meter_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Result

Bootstrap data now enforces a stronger token-usage replay contract:

- billing token counts cannot exist without corresponding metering token evidence
- token input and output quantities cannot silently drift between metering and billing
- seeded request evidence remains commercially replayable at the token-accounting level

This improves deployability for billing forensics, provider invoice replay, token usage auditability, and commercial seed-data trustworthiness.
