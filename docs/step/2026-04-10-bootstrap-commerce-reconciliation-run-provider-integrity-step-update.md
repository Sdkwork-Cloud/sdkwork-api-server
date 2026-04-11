# 2026-04-10 Bootstrap Commerce Reconciliation Run Provider Integrity Step Update

## What Changed

- Hardened bootstrap validation for `commerce_reconciliation_items`.
- A reconciliation item now fails bootstrap when its linked business objects drift outside the linked reconciliation run context:
  - referenced `payment_attempt.provider` must match `reconciliation_run.provider`
  - when `reconciliation_run.payment_method_id` is set, referenced `payment_attempt.payment_method_id` must match it
  - referenced `refund.provider` must match `reconciliation_run.provider`
  - when `reconciliation_run.payment_method_id` is set, referenced `refund.payment_method_id` must match it

## Why This Matters

- `commerce_reconciliation_runs` are not generic bookkeeping buckets.
- They represent a provider-scoped and optionally payment-method-scoped settlement review window.
- Previous validation already ensured:
  - the run exists
  - linked order, payment attempt, and refund records exist
  - attempt and refund records point back to the same order lineage
- That still left a finance-integrity gap:
  - a Stripe payment attempt or refund could be attached to a bank-transfer reconciliation run and still pass bootstrap
- In a commercial system, that would corrupt:
  - operator review queues
  - provider settlement drill-down
  - payment-method-specific reconciliation dashboards

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger reconciliation contract.
- This step makes an already-valid semantic rule explicit, so bootstrap rejects future seed drift before it reaches runtime or storage.

## Test Coverage Added

- reconciliation rejects a `payment_attempt` that falls outside the linked run provider/payment-method context
- reconciliation rejects a `refund` that falls outside the linked run provider/payment-method context

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_commerce_reconciliation_item_with -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- Future reconciliation seeds should continue to treat `reconciliation_run.provider` as the authoritative provider scope for all attached discrepancy records.
- If a later product requirement needs cross-provider reconciliation views, model that as a separate aggregate instead of weakening `commerce_reconciliation_runs`.
- If a run intentionally spans multiple payment methods, keep `payment_method_id = null`; otherwise use the field as a strict narrowing key.
