# 2026-04-11 Bootstrap Commerce Reconciliation Scope Closure Step Update

## What Changed

- Hardened bootstrap validation for `commerce_reconciliation_items`.
- Added reconciliation scope invariants:
  - linked `payment_attempt.updated_at_ms` must fall within the parent reconciliation run scope window
  - linked `refund.created_at_ms` must fall within the parent reconciliation run scope window
  - `external_reference` must resolve to at least one `commerce_payment_event.dedupe_key`
  - every payment event resolved through `external_reference` must stay within the reconciliation run provider context and scope window

## Why This Matters

- Reconciliation seed data is meant to be commercially actionable, not just cross-table linked.
- Previous validation already ensured:
  - referenced attempts and refunds belong to the same order when both are linked
  - attempts and refunds stay inside the reconciliation run provider/payment-method context
  - item detail payload shape is valid
- That still left silent drift paths:
  - a reconciliation item could point to an attempt or refund that happened outside the run's declared scope
  - an `external_reference` could drift away from normalized payment-event evidence
  - an external reference could still resolve to provider evidence outside the run scope
- In a commercial bootstrap pack, those gaps make seeded reconciliation evidence less trustworthy for finance review and operator debugging.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/commerce/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod ATTEMPT_WINDOW_MISS=0 REFUND_WINDOW_MISS=0 EVENT_WINDOW_MISS=0`
  - `PROFILE=dev ATTEMPT_WINDOW_MISS=0 REFUND_WINDOW_MISS=0 EVENT_WINDOW_MISS=0`
  - `PROFILE=prod EXTERNAL_REF_MISSING_EVENT=0 EXTERNAL_REF_PROVIDER_MISMATCH=0 EXTERNAL_REF_EVENT_WINDOW_MISS=0`
  - `PROFILE=dev EXTERNAL_REF_MISSING_EVENT=0 EXTERNAL_REF_PROVIDER_MISMATCH=0 EXTERNAL_REF_EVENT_WINDOW_MISS=0`

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger reconciliation scope contract.
- This step turns the current reconciliation walkthrough posture into an explicit bootstrap guarantee.

## Test Coverage Added

- reconciliation item rejects linked payment attempt outside run scope
- reconciliation item rejects linked refund outside run scope
- reconciliation item rejects external reference missing payment-event evidence
- reconciliation item rejects external-reference payment event outside run scope

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_commerce_reconciliation_item -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- Future reconciliation seeds should continue treating `external_reference` as normalized provider evidence rather than a free-form note field.
- If the product later needs free-form reconciliation annotations, add a distinct field instead of weakening `external_reference` closure.
