# 2026-04-10 Bootstrap Request Settlement Temporal Lineage Step Update

## What Changed

- Hardened bootstrap validation for `request_settlements` temporal lineage against the linked `request_meter_fact`.
- A request settlement now fails bootstrap when:
  - `created_at_ms < request_meter_fact.started_at_ms`
  - `settled_at_ms < request_meter_fact.finished_at_ms` when the request meter fact has `finished_at_ms` and the settlement is marked settled

## Why This Matters

- `request_meter_fact` is the canonical execution-side timing record for a billed request.
- `request_settlement` is the downstream accounting projection derived from that same request.
- Previous validation already ensured:
  - settlement ownership matches the linked request meter fact
  - settlement accounting fields line up with request-meter-fact accounting outputs
  - settlement local timestamps are internally ordered
- That still left a temporal-lineage gap:
  - a settlement could be created before the request even started
  - a settlement could be marked settled before the request finished
- In a commercial bootstrap pack, those sequences would break auditability for billing evidence, settlement review, and any future lifecycle analytics built on request execution chronology.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/request-metering/*.json`
  - `data/request-settlements/*.json`
  - `data/account-holds/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod CREATED_BEFORE_FACT_START=0`
  - `PROFILE=prod SETTLED_BEFORE_FACT_FINISH=0`
  - `PROFILE=prod CREATED_BEFORE_HOLD_CREATED=0`
  - `PROFILE=prod SETTLED_BEFORE_HOLD_UPDATE=0`
  - `PROFILE=dev CREATED_BEFORE_FACT_START=0`
  - `PROFILE=dev SETTLED_BEFORE_FACT_FINISH=0`
  - `PROFILE=dev CREATED_BEFORE_HOLD_CREATED=0`
  - `PROFILE=dev SETTLED_BEFORE_HOLD_UPDATE=0`
- This round only promoted the request-meter-fact lineage guarantees into bootstrap validation because that is the authoritative execution timeline already modeled by the billing domain.

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger settlement temporal-lineage contract.
- This step makes an already-valid execution-to-settlement chronology explicit so future seed updates cannot introduce pre-request or pre-finish settlement drift.

## Test Coverage Added

- request settlement rejects `created_at_ms` earlier than the linked request meter fact `started_at_ms`
- request settlement rejects `settled_at_ms` earlier than the linked request meter fact `finished_at_ms`

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_request_settlement_created_before_request_started -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_request_settlement_settled_before_request_finished -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- The next settlement hardening pass can evaluate whether `account_holds.created_at_ms` and `account_holds.updated_at_ms` should also become explicit lower bounds for settlement lifecycle timestamps.
- If future billing semantics require pre-created draft settlement records, that should be modeled explicitly with a separate lifecycle state instead of weakening execution-lineage timing guarantees.
