# 2026-04-10 Bootstrap Commerce Webhook Payment Method Lineage Step Update

## What Changed

- Hardened bootstrap validation for `commerce_webhook_inbox_records.payment_method_id`.
- When a webhook inbox record links to one or more `commerce_payment_events` through the same `provider_event_id`, and the linked order declares `payment_method_id`, the webhook inbox payment method must match that order payment method exactly.

## Why This Matters

- Previous hardening already guaranteed provider-level lineage:
  - webhook provider matches payment method provider
  - webhook provider matches linked payment event provider
- That still left a method-level gap:
  - a webhook could stay on the correct provider but drift to a different payment method under that same provider
- In a commercial system, that would weaken:
  - payment-method-specific webhook diagnostics
  - checkout-channel reporting
  - operator troubleshooting for provider flows that expose multiple payment methods on the same provider

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/commerce/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod WEBHOOK_EVENT_ORDER_PM_BAD=0`
  - `PROFILE=dev WEBHOOK_EVENT_ORDER_PM_BAD=0`
- Real linked coverage:
  - `PROFILE=prod MATCHED=4`
  - `PROFILE=dev MATCHED=6`

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger webhook payment-method lineage contract.
- This step promotes an already-valid data assumption into an explicit bootstrap invariant.

## Test Coverage Added

- webhook inbox rejects payment-method drift from the linked payment-event order, even when the alternative payment method stays on the same provider

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_commerce_webhook_inbox_with_payment_method_mismatched_linked_order -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- Future webhook seeds should continue to treat `payment_method_id` as a precise channel lineage field, not just a provider hint.
- If future product requirements introduce cross-method provider events, they should be modeled explicitly instead of weakening current webhook lineage guarantees.
