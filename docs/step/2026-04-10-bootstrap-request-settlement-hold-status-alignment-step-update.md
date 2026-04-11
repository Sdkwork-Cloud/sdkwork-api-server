# 2026-04-10 Bootstrap Request Settlement Hold Status Alignment Step Update

## What Changed

- Hardened bootstrap validation for `request_settlements` against their linked `account_holds`.
- Added two status-alignment invariants, scoped only to statuses already exercised by merged repository seed data:
  - `request_settlement.status = pending` now requires linked `hold.status = held`
  - `request_settlement.status = partially_released` now requires linked `hold.status = partially_released`

## Why This Matters

- Earlier validation already guaranteed:
  - settlement and hold identity linkage
  - quantity conservation across estimated, captured, and released values
  - temporal lineage between settlement and hold timestamps
- That still allowed a semantic gap where settlement lifecycle status could disagree with the hold posture it claimed to settle.
- For commercial bootstrap data, that creates operator confusion:
  - a `pending` settlement would imply an unsettled reservation while the linked hold could already be in a released or partial state
  - a `partially_released` settlement could point at a hold that was never partially released at all
- This step closes that gap without inventing broader rules for statuses that current merged bootstrap data does not yet materially exercise.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using:
  - `data/profiles/{prod,dev}.json`
  - ordered `updates/*` references
  - last-wins collapse by `hold_id` and `request_settlement_id`
- Audit result:
  - `PROFILE=prod PENDING_HOLD_NOT_HELD=0`
  - `PROFILE=prod PARTIAL_HOLD_NOT_PARTIAL=0`
  - `PROFILE=prod NULL_HOLD_PENDING=0`
  - `PROFILE=prod NULL_HOLD_PARTIAL=0`
  - `PROFILE=prod STATUS_HOLD_MATRIX={"partially_released::partially_released":21}`
  - `PROFILE=dev PENDING_HOLD_NOT_HELD=0`
  - `PROFILE=dev PARTIAL_HOLD_NOT_PARTIAL=0`
  - `PROFILE=dev NULL_HOLD_PENDING=0`
  - `PROFILE=dev NULL_HOLD_PARTIAL=0`
  - `PROFILE=dev STATUS_HOLD_MATRIX={"partially_released::partially_released":23,"pending::held":1}`
- Current merged bootstrap data therefore already treats these two settlement statuses as hold-state-specific postures rather than loose labels.

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger settlement-to-hold status contract.
- This round stays intentionally narrow:
  - no new semantics were forced for `captured`
  - no new semantics were forced for `released`
  - no new semantics were forced for `refunded`

## Test Coverage Added

- pending settlement rejects linked hold with non-`held` status
- partially released settlement rejects linked hold with non-`partially_released` status

## Verification

- `cargo test -p sdkwork-api-app-runtime request_settlement_with_non_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- The next safe candidate remains stricter posture semantics for settlement statuses such as `captured`, `released`, and `refunded`, but only after merged bootstrap data materially exercises those states.
- Until then, keeping the invariant surface limited to observed repository seed behavior avoids overfitting validation rules ahead of real data.
