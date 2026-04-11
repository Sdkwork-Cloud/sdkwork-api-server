# 2026-04-10 Bootstrap Commerce Payment Event Provider Lineage Step Update

## What Changed

- Hardened bootstrap validation for `commerce_payment_events`.
- A payment event now fails bootstrap unless its `provider` stays inside the linked order payment lineage:
  - when `order.payment_method_id` is set, `payment_event.provider` must match that payment method's provider
  - when `order.latest_payment_attempt_id` is set, `payment_event.provider` must match that latest payment attempt's provider

## Why This Matters

- `commerce_payment_events` are provider-originated settlement and webhook-processing facts.
- Previous validation already ensured:
  - the linked order exists
  - the linked project and user match the order
  - the payload shape is valid JSON
- That still left a lineage hole:
  - a Stripe-backed order could carry a `bank_transfer` payment event and still pass bootstrap
- In a commercial system, that would pollute:
  - provider settlement timelines
  - payment-event drill-down in admin/portal
  - downstream reconciliation and operator review surfaces

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/payment-methods/*.json`
  - `data/commerce/*.json`
  - `data/updates/*.json`
- Audit result:
  - `PROFILE=prod EVENT_ORDER_PM_PROVIDER_BAD=0`
  - `PROFILE=prod EVENT_ORDER_LATEST_ATTEMPT_PROVIDER_BAD=0`
  - `PROFILE=dev EVENT_ORDER_PM_PROVIDER_BAD=0`
  - `PROFILE=dev EVENT_ORDER_LATEST_ATTEMPT_PROVIDER_BAD=0`

## Data Impact

- No `/data` seed files required changes.
- Existing production and development bootstrap packs already satisfy the stronger payment-event lineage contract.
- This step converts an already-true repository invariant into an explicit bootstrap guarantee.

## Test Coverage Added

- payment event rejects provider drift from the linked order payment method
- payment event rejects provider drift from the linked order latest payment attempt

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_commerce_payment_event_with_provider_mismatched -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- Future payment-event seeds should continue to model `provider` as an exact lineage field, not as a loose annotation.
- If a workflow later needs cross-provider settlement correlation, model that separately instead of weakening `commerce_payment_events`.
- The next commerce hardening pass can safely evaluate:
  - `refund.payment_method_id -> provider` explicit consistency
  - `webhook_inbox.provider_event_id -> payment_event.provider` linkage invariants
