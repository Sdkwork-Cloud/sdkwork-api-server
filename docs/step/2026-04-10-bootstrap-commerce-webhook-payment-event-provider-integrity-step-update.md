# 2026-04-10 Bootstrap Commerce Webhook Payment Event Provider Integrity Step Update

## What Changed

- Hardened bootstrap validation for `commerce_webhook_inbox_records`.
- When a webhook inbox record declares `provider_event_id`, any linked `commerce_payment_event` sharing that same `provider_event_id` must use the same `provider`.

## Why This Matters

- `commerce_webhook_inbox_records` are the raw ingress trail for provider webhooks.
- `commerce_payment_events` are the normalized provider event facts derived from that ingress.
- Previous validation already ensured:
  - webhook payload shape is valid
  - webhook provider matches its payment method when `payment_method_id` is present
  - `provider_event_id` is non-empty when declared
- That still allowed a subtle lineage failure:
  - a `bank_transfer` webhook inbox record could reuse the same `provider_event_id` as an existing Stripe payment event and still pass bootstrap if `payment_method_id` was omitted
- In a commercial bootstrap pack, that would make provider-event drill-down and webhook reconciliation less trustworthy.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/commerce/*.json`
  - `data/payment-methods/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod WEBHOOK_EVENT_PROVIDER_BAD=0`
  - `PROFILE=dev WEBHOOK_EVENT_PROVIDER_BAD=0`
- Coverage of real linked cases:
  - `PROFILE=prod MATCHED_PROVIDER_EVENT_IDS=4`
  - `PROFILE=dev MATCHED_PROVIDER_EVENT_IDS=6`

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger webhook-to-payment-event provider lineage contract.
- This step turns an existing runtime expectation into an explicit bootstrap invariant.

## Test Coverage Added

- webhook inbox rejects provider drift from linked payment events that share `provider_event_id`

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_commerce_webhook_inbox_with_provider_mismatched_linked_payment_event -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- Future webhook seeds should continue to model `provider_event_id` as a provider-scoped identifier, not a cross-provider correlation key.
- If the product later needs cross-provider webhook correlation, introduce a separate normalized aggregate instead of weakening `commerce_webhook_inbox_records`.
- The next commerce hardening pass can evaluate refund-only lineage gaps that currently remain implicit when future seeds introduce refunds without payment attempts.
