# 2026-04-10 Bootstrap Commerce Payment Lifecycle Timestamp Closure Step Update

## What Changed

- Hardened bootstrap lifecycle validation for `commerce_payment_events`, `commerce_webhook_inbox_records`, `commerce_refunds`, and `commerce_reconciliation_runs`.
- Added status-driven timestamp invariants:
  - `commerce_payment_events.processing_status = processed` must declare `processed_at_ms`
  - `commerce_webhook_inbox_records.processing_status = processed` must declare `processed_at_ms`
  - `commerce_webhook_inbox_records.processed_at_ms` cannot be earlier than `last_received_at_ms`
  - `commerce_refunds.status = succeeded` must declare `completed_at_ms`
  - `commerce_reconciliation_runs.status = completed` must declare `completed_at_ms`

## Why This Matters

- The bootstrap pack is supposed to be commercially operable evidence, not just object linkage.
- Previous validation already covered basic timestamp ordering:
  - event processed time could not be earlier than event received time
  - webhook processed time could not be earlier than first receive time
  - refund and reconciliation completion times, if present, could not be earlier than creation
- That still left lifecycle gaps:
  - a processed event or processed webhook could omit the time when processing actually completed
  - a webhook could claim it was processed before the last successful receive attempt
  - a succeeded refund could omit its completion timestamp
  - a completed reconciliation run could omit its completion timestamp
- Those gaps weaken operator trust in seeded payment incident review, webhook replay reasoning, refund audits, and reconciliation history.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/commerce/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod EVENT_PROCESSED_MISSING_TS=0 WEBHOOK_PROCESSED_MISSING_TS=0 REFUND_SUCCEEDED_MISSING_TS=0 RUN_COMPLETED_MISSING_TS=0`
  - `PROFILE=dev EVENT_PROCESSED_MISSING_TS=0 WEBHOOK_PROCESSED_MISSING_TS=0 REFUND_SUCCEEDED_MISSING_TS=0 RUN_COMPLETED_MISSING_TS=0`
  - `PROFILE=prod WEBHOOK_PROCESSED_BEFORE_LAST=0`
  - `PROFILE=dev WEBHOOK_PROCESSED_BEFORE_LAST=0`

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger lifecycle timestamp contract.
- This step converts existing commercial data posture into explicit bootstrap guarantees before future updates can drift.

## Test Coverage Added

- processed payment event rejects missing `processed_at_ms`
- processed webhook inbox record rejects missing `processed_at_ms`
- webhook inbox record rejects `processed_at_ms < last_received_at_ms`
- succeeded refund rejects missing `completed_at_ms`
- completed reconciliation run rejects missing `completed_at_ms`

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_commerce_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- Future payment lifecycle seeds should keep status and timestamps as a single source of operational truth rather than relying on dashboard convention.
- If the product later introduces richer intermediate statuses, extend these lifecycle rules explicitly instead of weakening the existing processed/completed contracts.
