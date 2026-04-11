# 2026-04-10 Bootstrap Commerce Refund Payment Method Provider Integrity Step Update

## What Changed

- Hardened bootstrap validation for `commerce_refunds`.
- A refund now fails bootstrap when its declared `payment_method_id` belongs to a different provider than `commerce_refunds.provider`.
- This check now applies even when `payment_attempt_id` is absent.

## Why This Matters

- `commerce_refunds` are provider-settlement facts, not generic financial adjustments.
- Previous validation already ensured:
  - the order exists
  - the refund amount does not exceed the order payable amount
  - if `payment_attempt_id` is present, the refund stays on the same order and provider as that attempt
  - if both `payment_attempt_id` and `payment_method_id` are present, the payment method matches the attempt
- That still left one lineage gap:
  - a refund could omit `payment_attempt_id`, keep a Stripe payment method id, declare `provider = bank_transfer`, and still pass bootstrap
- In a commercial bootstrap pack, that would weaken refund drill-down, provider finance review, and downstream reconciliation trust.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/commerce/*.json`
  - `data/payment-methods/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod REFUND_PM_PROVIDER_BAD=0`
  - `PROFILE=dev REFUND_PM_PROVIDER_BAD=0`
- Current repository shape:
  - `PROFILE=prod REFUNDS_WITH_PM_AND_NO_ATTEMPT=0`
  - `PROFILE=dev REFUNDS_WITH_PM_AND_NO_ATTEMPT=0`

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger refund lineage contract.
- This step turns an existing commercial assumption into an explicit bootstrap invariant before future seeds can drift.

## Test Coverage Added

- refund rejects payment-method/provider drift when no payment attempt is linked

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_commerce_refund_with_payment_method_provider_mismatch_without_payment_attempt -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- Future refund seeds may still omit `payment_attempt_id` if the domain later needs that flexibility, but they must stay consistent with the declared payment method provider.
- If future commercial flows introduce manual refunds with no payment attempt, keep provider lineage explicit instead of relying on operator convention.
- The next commerce hardening pass can evaluate whether refund currency and order currency should also be modeled as an explicit bootstrap invariant.
