# 2026-04-10 Bootstrap Account Hold Request Metering Linkage Step Update

## What Changed

- Hardened bootstrap validation for `account_holds` against their linked `request_meter_facts`.
- Added explicit linkage invariants for every hold that references `request_id`:
  - the request meter fact must exist
  - hold tenant, organization, user, and account ownership must match the fact
  - `estimated_quantity` must match `estimated_credit_hold`
  - `status = held` requires `usage_capture_status = estimated`
  - `status = partially_released` requires `usage_capture_status in {captured, reconciled}`
  - `created_at_ms` must not be earlier than fact `started_at_ms`
  - partially released holds must not be updated before fact `finished_at_ms` when finish time exists

## Why This Matters

- `account_holds` and `request_meter_facts` describe the same commercial request lifecycle from two different kernels:
  - metering states what usage was observed
  - account holds state what credit was reserved, captured, or released for that usage
- Earlier validation already protected quantity posture and settlement arithmetic, but it still allowed the two records to drift as long as each record looked internally valid.
- In a commercial bootstrap pack, that would let operators seed:
  - a hold attached to a non-existent request fact
  - a hold owned by a different account than the metering fact
  - a held reservation paired with a captured metering record
- That kind of drift breaks downstream reconciliation, ledger evidence, and request-level billing drill-down.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/account-holds/*.json`
  - `data/request-metering/*.json`
- Audit result:
  - `PROFILE=prod MISSING_FACTS=0`
  - `PROFILE=prod OWNERSHIP_DRIFT=0`
  - `PROFILE=prod ESTIMATED_DRIFT=0`
  - `PROFILE=prod CREATED_BEFORE_STARTED=0`
  - `PROFILE=prod UPDATED_BEFORE_STARTED=0`
  - `PROFILE=prod HELD_FACT_NOT_ESTIMATED=0`
  - `PROFILE=prod PARTIAL_FACT_NOT_CAPTURED=0`
  - `PROFILE=prod MATRIX={"partially_released::captured":21}`
  - `PROFILE=dev MISSING_FACTS=0`
  - `PROFILE=dev OWNERSHIP_DRIFT=0`
  - `PROFILE=dev ESTIMATED_DRIFT=0`
  - `PROFILE=dev CREATED_BEFORE_STARTED=0`
  - `PROFILE=dev UPDATED_BEFORE_STARTED=0`
  - `PROFILE=dev HELD_FACT_NOT_ESTIMATED=0`
  - `PROFILE=dev PARTIAL_FACT_NOT_CAPTURED=0`
  - `PROFILE=dev MATRIX={"held::estimated":1,"partially_released::captured":23}`

## Data Impact

- No `/data` seed files required changes for this linkage hardening.
- Existing merged bootstrap data already encoded consistent hold-to-metering lineage.
- This step turns that already-observed commercial discipline into a hard bootstrap contract.

## Test Coverage Added

- hold rejects missing request meter fact
- hold rejects ownership drift against request meter fact
- hold rejects estimated quantity drift against request meter fact
- held hold rejects captured request meter fact
- partially released hold rejects estimated request meter fact
- hold rejects creation before request start
- partially released hold rejects update before request finish

## Verification

- `cargo test -p sdkwork-api-app-runtime account_hold_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- The next safe hardening pass is to extend request-fact linkage to any future hold states only after repository seed data materially exercises them.
- If future bootstrap packs introduce `captured`, `released`, `expired`, or `failed` holds, keep the same rule: encode the metering posture explicitly and make bootstrap reject drift immediately.
