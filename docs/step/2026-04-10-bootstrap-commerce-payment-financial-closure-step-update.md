# 2026-04-10 Bootstrap Commerce Payment Financial Closure Step Update

## What Changed

- Hardened bootstrap validation for `commerce_orders`, `commerce_payment_attempts`, `commerce_payment_events`, and `commerce_webhook_inbox_records`.
- Added financial closure invariants:
  - `commerce_orders.latest_payment_attempt_id` must point to an attempt whose `amount_minor` matches `payable_price_cents`
  - `commerce_orders.refunded_amount_minor` must equal the sum of linked `commerce_refunds.amount_minor`
  - `commerce_payment_attempts.refunded_amount_minor` must equal the sum of linked `commerce_refunds.amount_minor`
  - `commerce_payment_attempts.status = succeeded` now requires both full capture and `completed_at_ms`
- Added event lineage invariants:
  - `commerce_payment_events.processed_at_ms` cannot be earlier than `received_at_ms`
  - `commerce_webhook_inbox_records.dedupe_key` must match any linked `commerce_payment_event` sharing the same `provider_event_id`

## Why This Matters

- The bootstrap pack is meant to be install-ready commercial truth, not just syntactically valid seed data.
- Previous validation already covered existence, provider alignment, currency alignment, refund overflow, and webhook/provider linkage.
- That still left a few silent drift paths:
  - an order could point at a latest payment attempt whose amount no longer matched the order payable amount
  - order and attempt refund totals could drift from the actual refund facts
  - a succeeded payment attempt could look complete without being fully captured or timestamped as completed
  - normalized payment events could claim impossible processing order
  - webhook dedupe evidence could drift from the normalized payment event derived from the same provider event
- These gaps weaken finance review, refund reconciliation, payment debugging, and admin trust in seeded commercial walkthroughs.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/commerce/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod LATEST_ATTEMPT_AMOUNT_MISMATCH=0 ORDER_REFUND_SUM_MISMATCH=0 ATTEMPT_REFUND_SUM_MISMATCH=0 SUCCEEDED_CAPTURE_MISMATCH=0 SUCCEEDED_COMPLETION_MISSING=0 EVENT_PROCESSED_BEFORE_RECEIVED=0 WEBHOOK_DEDUPE_MISMATCH=0`
  - `PROFILE=dev LATEST_ATTEMPT_AMOUNT_MISMATCH=0 ORDER_REFUND_SUM_MISMATCH=0 ATTEMPT_REFUND_SUM_MISMATCH=0 SUCCEEDED_CAPTURE_MISMATCH=0 SUCCEEDED_COMPLETION_MISSING=0 EVENT_PROCESSED_BEFORE_RECEIVED=0 WEBHOOK_DEDUPE_MISMATCH=0`

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger payment-financial closure contract.
- Local runtime test fixtures were normalized so the bootstrap test harness models the same refund totals as the repository seed data.

## Test Coverage Added

- order rejects latest payment attempt amount drift from payable price
- order rejects refunded total drift from linked refunds
- payment attempt rejects refunded total drift from linked refunds
- succeeded payment attempt rejects partial capture
- succeeded payment attempt rejects missing completion timestamp
- payment event rejects `processed_at_ms < received_at_ms`
- webhook inbox rejects dedupe drift from linked payment event

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_commerce_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- Future commercial seeds should continue treating order totals, attempt totals, refund facts, and webhook/event dedupe evidence as one closed accounting graph.
- If the product later supports split-tender or multi-attempt refund attribution, add an explicit aggregate model rather than weakening these bootstrap invariants implicitly.
