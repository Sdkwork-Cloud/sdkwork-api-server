# 2026-04-10 Bootstrap Request Settlement Lifecycle Posture Step Update

## What Changed

- Hardened bootstrap validation for `request_settlements` lifecycle posture.
- A request settlement now fails bootstrap when:
  - `status = pending` and `settled_at_ms != 0`
  - `status = pending` and any realized accounting field is non-zero:
    - `released_credit_amount`
    - `captured_credit_amount`
    - `provider_cost_amount`
    - `retail_charge_amount`
    - `shortfall_amount`
    - `refunded_amount`
  - `status` is one of:
    - `captured`
    - `partially_released`
    - `released`
    - `refunded`
    and `settled_at_ms == 0`

## Why This Matters

- `request_settlement.status` is not just display metadata. It is the lifecycle posture for account-kernel settlement evidence.
- Previous validation already ensured:
  - settlement accounting values are finite and non-negative
  - captured/released/refunded conservation holds
  - settlement timing stays ordered against itself, the linked request meter fact, and the linked hold
  - settlement quantities line up with hold and metering evidence
- That still left a lifecycle gap:
  - a `pending` settlement could look structurally valid while already carrying realized accounting or a completion timestamp
  - a completed settlement posture could omit `settled_at_ms`
- In commercial bootstrap data, those records weaken billing replay, operator trust, and any admin or portal surface that relies on status to summarize settlement progress.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/request-settlements/*.json`
  - `data/request-metering/*.json`
  - additive `data/updates/*.json`
- Audit result:
  - `PROFILE=prod PENDING_SETTLED_NON_ZERO=0`
  - `PROFILE=prod PENDING_NON_ZERO_ACCOUNTING=0`
  - `PROFILE=prod NON_PENDING_ZERO_SETTLED=0`
  - `PROFILE=prod PARTIALLY_RELEASED_MISSING_AMOUNTS=0`
  - `PROFILE=prod PARTIALLY_RELEASED_MISSING_SETTLED=0`
  - `PROFILE=dev PENDING_SETTLED_NON_ZERO=0`
  - `PROFILE=dev PENDING_NON_ZERO_ACCOUNTING=0`
  - `PROFILE=dev NON_PENDING_ZERO_SETTLED=0`
  - `PROFILE=dev PARTIALLY_RELEASED_MISSING_AMOUNTS=0`
  - `PROFILE=dev PARTIALLY_RELEASED_MISSING_SETTLED=0`
- This step only promoted the minimal lifecycle posture guarantees. The stricter partially-released semantic checks were left for a later pass.

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger lifecycle-posture contract.
- This step makes settlement status semantics explicit so future update packs cannot silently introduce contradictory posture records.

## Test Coverage Added

- pending request settlement rejects `settled_at_ms`
- pending request settlement rejects realized accounting values
- non-pending completed settlement posture rejects missing `settled_at_ms`

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_pending_request_settlement -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_non_pending_request_settlement_without_settled_timestamp -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- The next safe settlement candidate is stricter status-to-amount semantics for `partially_released`, because merged `prod/dev` data already shows:
  - `released_credit_amount > 0`
  - `captured_credit_amount > 0`
  - `settled_at_ms > 0`
- That candidate was audited in this round but not promoted yet.
