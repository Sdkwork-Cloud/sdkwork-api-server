# 2026-04-10 Bootstrap Account Hold Status Posture Step Update

## What Changed

- Hardened bootstrap validation for `account_holds` based on hold status posture.
- Added status-specific invariants, limited to hold statuses already exercised by merged repository seed data:
  - `account_hold.status = held` now requires zero realized quantities
  - `account_hold.status = partially_released` now requires `captured_quantity > 0`
  - `account_hold.status = partially_released` now requires `released_quantity > 0`

## Why This Matters

- Earlier validation already guaranteed:
  - hold ownership matches the linked account
  - quantities are finite and non-negative
  - captured plus released does not exceed estimated
  - hold allocation totals reconcile back to the parent hold
- That still left `status` as a weak label rather than a real accounting posture.
- In commercial bootstrap data, operators read hold state as an account-kernel fact:
  - `held` means credit is still fully reserved
  - `partially_released` means some credit was captured and some was released
- Without this invariant, seed data could claim one posture while encoding another, and downstream settlement validation would be forced to absorb inconsistent state later in the lifecycle.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using:
  - `data/profiles/{prod,dev}.json`
  - ordered `updates/*` references
  - last-wins collapse by `hold_id`
- Audit result:
  - `PROFILE=prod HELD_NON_ZERO=0`
  - `PROFILE=prod PARTIAL_NO_CAPTURE=0`
  - `PROFILE=prod PARTIAL_NO_RELEASE=0`
  - `PROFILE=prod MATRIX={"partially_released::cap=1::rel=1":21}`
  - `PROFILE=dev HELD_NON_ZERO=0`
  - `PROFILE=dev PARTIAL_NO_CAPTURE=0`
  - `PROFILE=dev PARTIAL_NO_RELEASE=0`
  - `PROFILE=dev MATRIX={"held::cap=0::rel=0":1,"partially_released::cap=1::rel=1":23}`
- Current merged bootstrap data already treats these hold statuses as strict quantity postures rather than descriptive hints.

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger hold posture contract.
- Two older settlement-status regression fixtures were adjusted to keep targeting settlement-to-hold status drift specifically after hold posture became stricter:
  - pending settlement mismatch fixture now uses a non-`held` hold status that remains internally valid
  - partially released settlement mismatch fixture now uses a non-`partially_released` hold status that remains internally valid

## Test Coverage Added

- held account hold rejects realized quantity
- partially released account hold rejects missing captured quantity
- partially released account hold rejects missing released quantity

## Verification

- `cargo test -p sdkwork-api-app-runtime account_hold_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime request_settlement_with_non_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- The next safe account-kernel candidates are status posture for other hold states such as `captured`, `released`, `expired`, and `failed`, but only after merged bootstrap data materially exercises those states.
- Until those statuses show up in repository seed data, keeping validation limited to observed posture semantics preserves rigor without overfitting.
