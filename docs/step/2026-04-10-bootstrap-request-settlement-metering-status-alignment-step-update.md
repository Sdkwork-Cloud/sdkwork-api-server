# 2026-04-10 Bootstrap Request Settlement Metering Status Alignment Step Update

## What Changed

- Hardened bootstrap validation for `request_settlements` against the linked `request_meter_fact.usage_capture_status`.
- A request settlement now fails bootstrap when:
  - `status = pending` and the linked request meter fact is not `estimated`
  - `status` is one of:
    - `captured`
    - `partially_released`
    - `released`
    - `refunded`
    and the linked request meter fact is not `captured` or `reconciled`

## Why This Matters

- `request_settlement.status` describes the account-kernel settlement lifecycle.
- `request_meter_fact.usage_capture_status` describes the execution-side billing capture lifecycle for the same request.
- Previous validation already ensured:
  - settlement ownership matches the linked request meter fact
  - settlement amounts align with metering-side accounting fields
  - settlement timestamps stay ordered against both metering and hold evidence
  - settlement lifecycle posture itself is internally coherent
- That still left a cross-domain lifecycle gap:
  - a pending settlement could reference a request fact that was already captured
  - a completed settlement could reference a request fact that was still only estimated
- In commercial bootstrap data, that breaks the semantic contract that usage capture and settlement progression are two views of the same billing lifecycle.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/request-settlements/*.json`
  - `data/request-metering/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod PENDING_NOT_ESTIMATED=0`
  - `PROFILE=prod PENDING_HAS_ACTUAL=0`
  - `PROFILE=prod NON_PENDING_STILL_ESTIMATED=0`
  - `PROFILE=prod NON_PENDING_MISSING_ACTUAL=0`
  - `PROFILE=prod STATUS_CAPTURE_MATRIX={"partially_released::captured":21}`
  - `PROFILE=dev PENDING_NOT_ESTIMATED=0`
  - `PROFILE=dev PENDING_HAS_ACTUAL=0`
  - `PROFILE=dev NON_PENDING_STILL_ESTIMATED=0`
  - `PROFILE=dev NON_PENDING_MISSING_ACTUAL=0`
  - `PROFILE=dev STATUS_CAPTURE_MATRIX={"partially_released::captured":23,"pending::estimated":1}`
- Current repository bootstrap data already expresses one coherent lifecycle matrix:
  - pending settlement -> estimated metering
  - completed settlement -> captured metering

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger settlement-to-metering lifecycle alignment contract.
- This step promotes an already-valid commercial assumption into explicit bootstrap validation so later seed updates cannot drift settlement status away from metering capture state.

## Test Coverage Added

- pending settlement rejects a linked request meter fact with `usage_capture_status = captured`
- completed settlement rejects a linked request meter fact with `usage_capture_status = estimated`

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_pending_request_settlement_with_captured_request_fact -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_completed_request_settlement_with_estimated_request_fact -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- The next safe settlement candidate is stricter `partially_released` amount posture:
  - `released_credit_amount > 0`
  - `captured_credit_amount > 0`
  - `refunded_amount = 0`
- That candidate was re-audited in this round and still looks clean in merged bootstrap data, but it was not promoted in this step.
