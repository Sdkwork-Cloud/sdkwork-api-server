# 2026-04-10 Bootstrap Request Metering Timing Accounting Closure Step Update

## Summary

This step hardens another part of the commercial bootstrap evidence chain around request metering lifecycle integrity:

- previous slices already required request metering to resolve to valid billing, account, pricing-plan, and workspace evidence
- but bootstrap still allowed request usage facts and request usage metrics to drift on lifecycle timing and capture-state accounting

That left two bootstrap gaps:

- a metric could be captured after the parent request had already finished or been last updated
- a request could claim `captured` usage without actual accounting values, or claim `estimated` usage while already carrying actual accounting values

Both cases are structurally valid JSON, but commercially invalid seed evidence.

## What Changed

### 1. Added request-meter-fact capture-state accounting closure

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- `usage_capture_status = pending|estimated` must not set:
  - `actual_credit_charge`
  - `actual_provider_cost`
- `usage_capture_status = captured|reconciled` must require:
  - `actual_credit_charge`
  - `actual_provider_cost`
  - `finished_at_ms`

This makes bootstrap reject request facts that claim a lifecycle state inconsistent with their accounting evidence.

### 2. Added request-meter-metric parent timing-window closure

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- `request_meter_metrics.captured_at_ms` must not be later than the parent `request_meter_facts.finished_at_ms` when a finish time exists
- `request_meter_metrics.captured_at_ms` must not be later than the parent `request_meter_facts.updated_at_ms`

This prevents late metrics from being seeded against a request record that already claims to be complete.

### 3. Added regression coverage for timing and accounting drift

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Added regression tests for:

- metrics captured after the parent request finish window
- metrics captured after the parent request update window
- captured request facts missing actual accounting values
- estimated request facts that already contain actual accounting values

## Data Audit Result

One real `/data` drift was exposed and fixed.

While the new validation itself was correct, full bootstrap regression found that merged profile state for:

- `request_id = 610002`
- `request_id = 610003`

was inconsistent after `2026-04-global-pricing-governance-linkage` overrode request facts without preserving the original execution timing window used by previously seeded metrics.

Updated:

- `data/request-metering/2026-04-global-pricing-governance-linkage.json`

Fix:

- restored the `started_at_ms`, `finished_at_ms`, `created_at_ms`, and `updated_at_ms` values for the ERNIE and MiniMax request facts so they remain compatible with the already-seeded request metrics from `2026-04-global-asia-account-kernel-operations`

After that fix, merged `prod` and `dev` request-metering profiles were clean again.

## Verification

Passed:

- `cargo test -p sdkwork-api-app-runtime request_meter_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Result

Bootstrap data now enforces a stronger request-metering lifecycle contract:

- request facts cannot claim captured accounting without actual amounts
- estimated usage cannot silently carry finalized accounting
- request metrics cannot arrive after the parent request lifecycle has already closed
- merged profile updates must preserve timing compatibility with previously seeded metric evidence

This improves commercial deployability for request replay, billing forensics, lifecycle auditability, and update-pack safety.
