# 2026-04-10 Bootstrap Request Settlement Partially Released Posture Step Update

## What Changed

- Hardened bootstrap validation for `request_settlements` when `status = partially_released`.
- A partially released settlement now fails bootstrap when:
  - `released_credit_amount <= 0`
  - `captured_credit_amount <= 0`
  - `refunded_amount > 0`

## Why This Matters

- `partially_released` is not a cosmetic label. It means the original hold has been split into:
  - a captured portion
  - a released portion
- Previous validation already ensured:
  - settlement lifecycle posture is coherent at a high level
  - settlement aligns with hold and metering quantities
  - settlement aligns with metering capture status
  - captured/released totals obey conservation rules
- That still left a status-specific semantic gap:
  - a settlement could claim `partially_released` while having no released portion
  - or no captured portion
  - or already carrying refunded credit without transitioning to `refunded`
- In commercial bootstrap data, that weakens operator trust because the status would no longer describe the accounting posture it claims to represent.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/request-settlements/*.json`
  - `data/request-metering/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod PARTIAL_RELEASED_NO_RELEASE=0`
  - `PROFILE=prod PARTIAL_RELEASED_NO_CAPTURE=0`
  - `PROFILE=prod PARTIAL_RELEASED_REFUND_NON_ZERO=0`
  - `PROFILE=dev PARTIAL_RELEASED_NO_RELEASE=0`
  - `PROFILE=dev PARTIAL_RELEASED_NO_CAPTURE=0`
  - `PROFILE=dev PARTIAL_RELEASED_REFUND_NON_ZERO=0`
- Current merged bootstrap data already uses `partially_released` as a strict account-kernel posture:
  - some credit captured
  - some credit released
  - no refund recorded yet

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger partially-released posture contract.
- One regression fixture was adjusted so the older `refunded_amount > captured_credit_amount` conservation test continues to target refund conservation specifically by using `status = refunded`.

## Test Coverage Added

- partially released settlement rejects missing released credit
- partially released settlement rejects missing captured credit
- partially released settlement rejects non-zero refunded credit

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_partially_released_request_settlement_without_released_amount -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_partially_released_request_settlement_without_captured_amount -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_partially_released_request_settlement_with_refunded_amount -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_request_settlement_with_refunded_amount_exceeding_captured_amount -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- The next safe request-settlement candidate is stricter status posture for `captured` and `refunded` once bootstrap data includes more real examples of those statuses.
- Today’s merged bootstrap packs only ship `pending` and `partially_released`, so this step stayed focused on the statuses that are already materially exercised by the repository seed data.
