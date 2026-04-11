# 2026-04-10 Bootstrap Request Settlement Hold Temporal Lineage Step Update

## What Changed

- Hardened bootstrap validation for `request_settlements` temporal lineage against the linked `account_hold`.
- A request settlement now fails bootstrap when:
  - `created_at_ms < account_hold.created_at_ms`
  - `updated_at_ms < account_hold.updated_at_ms`
  - `settled_at_ms < account_hold.updated_at_ms` when the settlement is marked settled

## Why This Matters

- `account_hold` is the canonical credit-reservation record for the request-side account kernel.
- `request_settlement` is the downstream settlement projection for that same hold-backed request.
- Previous validation already ensured:
  - settlement and hold agree on account/request identity
  - settlement estimated, captured, and released quantities match the linked hold
  - settlement local timestamps are internally ordered
  - settlement also respects request-meter-fact temporal lower bounds
- That still left a hold-lineage timing gap:
  - a settlement could be created before the hold existed
  - a settlement could report an earlier `updated_at_ms` than the hold state it claimed to mirror
  - a settlement could be marked settled before the linked hold reached its recorded updated state
- In commercial bootstrap data, those drifts weaken settlement replay, hold-release forensics, and operator trust in the seeded account-kernel evidence chain.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/account-holds/*.json`
  - `data/request-settlements/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod CREATED_BEFORE_HOLD_CREATED=0`
  - `PROFILE=prod UPDATED_BEFORE_HOLD_UPDATED=0`
  - `PROFILE=prod SETTLED_BEFORE_HOLD_UPDATED=0`
  - `PROFILE=dev CREATED_BEFORE_HOLD_CREATED=0`
  - `PROFILE=dev UPDATED_BEFORE_HOLD_UPDATED=0`
  - `PROFILE=dev SETTLED_BEFORE_HOLD_UPDATED=0`

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger settlement-to-hold temporal contract.
- This step promotes an already-valid account-kernel chronology into explicit bootstrap validation so future seed updates cannot backdate settlements relative to their linked holds.

## Test Coverage Added

- request settlement rejects `created_at_ms` earlier than linked hold `created_at_ms`
- request settlement rejects `updated_at_ms` earlier than linked hold `updated_at_ms`
- request settlement rejects `settled_at_ms` earlier than linked hold `updated_at_ms`

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_request_settlement_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- The next settlement hardening pass can safely evaluate status/lifecycle semantics now that hold-side temporal projection is explicit.
- Current audit candidates that already look clean in merged bootstrap data include pending-settlement zero-accounting posture and partially-released settlement status alignment, but they were not promoted in this step.
