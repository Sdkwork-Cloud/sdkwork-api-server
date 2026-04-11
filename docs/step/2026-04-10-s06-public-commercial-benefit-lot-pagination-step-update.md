# 2026-04-10 S06 Public Commercial Benefit-Lot Pagination Step Update

## Scope

- Step: `S06 Portal and Public API productization closure`
- Loop target: close the scale gap on `GET /commercial/account/benefit-lots`
- Boundaries:
  - `sdkwork-api-storage-core`
  - `sdkwork-api-storage-sqlite`
  - `sdkwork-api-storage-postgres`
  - `sdkwork-api-interface-http`

## Changes

- Storage contract:
  - added account-scoped cursor query:
    - `list_account_benefit_lots_for_account(account_id, after_lot_id, limit)`
- Storage runtime:
  - sqlite and postgres now read benefit lots by `account_id`
  - cursor semantics use `after_lot_id`
  - public page order is stable on `lot_id ASC`
- Storage schema:
  - added index:
    - `idx_ai_account_benefit_lot_account_lot`
- Public API:
  - `GET /commercial/account/benefit-lots` now accepts:
    - `after_lot_id`
    - `limit`
  - response now adds:
    - `page.limit`
    - `page.after_lot_id`
    - `page.next_after_lot_id`
    - `page.has_more`
    - `page.returned_count`
- Runtime posture:
  - removed store-wide lot listing plus in-memory account filtering from the public route

## Verification

- RED:
  - `cargo test -p sdkwork-api-storage-sqlite --test account_kernel_roundtrip -- --nocapture`
    - failed because `list_account_benefit_lots_for_account(...)` did not exist on the store surface
  - `cargo test -p sdkwork-api-interface-http --test market_coupon_commercial_routes -- --nocapture`
    - failed because `/commercial/account/benefit-lots` did not expose cursor pagination metadata
  - `cargo test -p sdkwork-api-interface-http --test openapi_route -- --nocapture`
    - failed because gateway OpenAPI did not publish `after_lot_id / limit` or the `page` schema
  - `cargo test -p sdkwork-api-storage-postgres --test account_kernel_pricing_surface -- --nocapture`
    - failed because postgres store surface did not implement the new account-scoped lot query
- GREEN:
  - `cargo test -p sdkwork-api-storage-sqlite --test account_kernel_roundtrip -- --nocapture`
  - `cargo test -p sdkwork-api-interface-http --test market_coupon_commercial_routes -- --nocapture`
  - `cargo test -p sdkwork-api-interface-http --test openapi_route -- --nocapture`
  - `cargo test -p sdkwork-api-storage-postgres --test account_kernel_pricing_surface -- --nocapture`
  - `cargo test -p sdkwork-api-storage-sqlite --test account_schema -- --nocapture`
  - `cargo test -p sdkwork-api-storage-postgres --test integration_postgres -- schema_and_accounts --nocapture`
  - `cargo test -p sdkwork-api-interface-http --test canonical_account_admission -- --nocapture`

## Result

- public commercial benefit-lot visibility is now account-scoped at the storage boundary instead of route-side filtering
- the outward contract now supports cursor pagination without weakening coupon/account-arrival semantics
- sqlite and postgres migrations now carry a dedicated account-lot index for this read path

## Exit

- Step result: `conditional-go`
- Reason:
  - the public scale gap on benefit-lot query/pagination/indexing is closed
  - downstream SDK/client regeneration against the updated public gateway schema is still the next `S06` follow-up
