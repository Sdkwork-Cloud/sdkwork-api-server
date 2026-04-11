# 2026-04-10 Bootstrap Commerce Currency Lineage Step Update

## What Changed

- Hardened bootstrap validation for:
  - `commerce_payment_attempts.currency_code`
  - `commerce_refunds.currency_code`
- Both records now fail bootstrap unless their `currency_code` matches the linked `commerce_order.currency_code`.

## Why This Matters

- Orders, payment attempts, and refunds are the same commercial money flow at different lifecycle stages.
- Previous validation already ensured:
  - the linked order exists
  - payment attempts and refunds stay on the same order lineage
  - refund amounts do not exceed the order payable amount
  - provider and payment-method lineage stays coherent
- That still allowed a finance-integrity gap:
  - an order could be denominated in `USD` while its attempt or refund claimed `CNY` and still pass bootstrap
- In a commercial-ready bootstrap pack, that would weaken settlement reporting, reconciliation, refund review, and any downstream revenue analytics keyed by order currency.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/commerce/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod PAYMENT_ATTEMPT_ORDER_CURRENCY_BAD=0`
  - `PROFILE=prod REFUND_ORDER_CURRENCY_BAD=0`
  - `PROFILE=dev PAYMENT_ATTEMPT_ORDER_CURRENCY_BAD=0`
  - `PROFILE=dev REFUND_ORDER_CURRENCY_BAD=0`

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger commerce currency-lineage contract.
- This step promotes an already-valid repository assumption into an explicit bootstrap invariant.

## Test Coverage Added

- payment attempt rejects currency drift from the linked order
- refund rejects currency drift from the linked order

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_commerce_payment_attempt_with_currency_mismatched_order -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_commerce_refund_with_currency_mismatched_order -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- Future commerce seeds should continue to model `currency_code` as a strict order-lineage field, not a loose display field.
- If future product requirements introduce cross-currency settlement or FX conversion, model that explicitly instead of weakening order-linked commerce records.
- The next commerce hardening pass can evaluate whether webhook payload metadata should also expose an explicit order-currency projection for audit surfaces.
