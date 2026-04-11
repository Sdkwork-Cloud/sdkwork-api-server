# 2026-04-10 Bootstrap Request Settlement Accounting Conservation Step Update

## What Changed

- Hardened bootstrap validation for `request_settlements`.
- A request settlement now fails bootstrap when:
  - `captured_credit_amount + released_credit_amount > estimated_credit_hold`
  - `refunded_amount > captured_credit_amount`

## Why This Matters

- `request_settlements` are the final commercial accounting projection for a metered request.
- Previous validation already ensured:
  - account ownership matches
  - linked request meter fact ownership matches
  - linked hold amounts match when `hold_id` is present
  - captured/provider amounts match request-meter fact accounting fields
- That still left two conservation gaps:
  - in a settlement without `hold_id`, captured and released credits could exceed the original estimated hold
  - refunded credit could exceed the total captured credit
- Both cases break basic accounting conservation and should never survive bootstrap.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/request-settlements/*.json`
  - `data/account-holds/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod RELEASED_GT_ESTIMATED=0`
  - `PROFILE=prod CAPTURED_GT_ESTIMATED=0`
  - `PROFILE=prod REFUND_GT_CAPTURED=0`
  - `PROFILE=dev RELEASED_GT_ESTIMATED=0`
  - `PROFILE=dev CAPTURED_GT_ESTIMATED=0`
  - `PROFILE=dev REFUND_GT_CAPTURED=0`
- Current repository shape:
  - `PROFILE=prod NO_HOLD=0`
  - `PROFILE=dev NO_HOLD=0`

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger settlement conservation contract.
- This step makes basic accounting invariants explicit before future seeds can introduce hold-less or over-refunded settlement drift.

## Test Coverage Added

- request settlement rejects captured+released totals that exceed the estimated hold, even when `hold_id` is null
- request settlement rejects refunded amounts that exceed captured credit

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_request_settlement_with_captured_and_released_exceeding_estimated_hold -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_request_settlement_with_refunded_amount_exceeding_captured_amount -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- Future settlement seeds may still omit `hold_id` if product requirements eventually need that flexibility, but they must continue to satisfy the same accounting conservation rules.
- The next billing hardening pass can evaluate whether `retail_charge_amount`, `provider_cost_amount`, and `shortfall_amount` should gain stricter semantic relationships once those financial semantics are modeled more explicitly.
