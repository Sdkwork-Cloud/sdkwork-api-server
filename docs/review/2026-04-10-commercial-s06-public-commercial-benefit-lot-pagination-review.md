# 2026-04-10 Commercial S06 Public Commercial Benefit-Lot Pagination Review

## Scope

- Architecture reference: `166`
- Step reference: `108`
- Loop focus: close public commercial benefit-lot scale drift after route convergence

## Findings

### P1 - public benefit-lot route still depended on store-wide listing

- `GET /commercial/account/benefit-lots` loaded all lots through `list_account_benefit_lots()`
- the route then filtered by `account_id` in memory
- impact:
  - cost scaled with total lot volume, not account volume
  - the route could not become a safe commercial history surface

### P1 - public contract had no cursor pagination despite the scale follow-up already being known

- the route returned a bare `benefit_lots` array with no page state
- impact:
  - large account histories had no bounded traversal contract
  - downstream clients could not iterate lot history safely

### P2 - billing schema had no dedicated index for account-lot traversal

- only `idx_ai_account_benefit_lot_account_status_expiry` existed
- impact:
  - the new public read path had no dedicated index aligned to `account_id + lot_id`

## Fix Closure

- added store-level account-scoped cursor query on both sqlite and postgres
- upgraded the public route to `after_lot_id + limit` cursor semantics with additive `page` metadata
- added `idx_ai_account_benefit_lot_account_lot` to both billing migrations
- synchronized gateway OpenAPI and route regressions with the new pagination contract

## Verification

- `cargo test -p sdkwork-api-storage-sqlite --test account_kernel_roundtrip -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test market_coupon_commercial_routes -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test openapi_route -- --nocapture`
- `cargo test -p sdkwork-api-storage-postgres --test account_kernel_pricing_surface -- --nocapture`
- `cargo test -p sdkwork-api-storage-sqlite --test account_schema -- --nocapture`
- `cargo test -p sdkwork-api-storage-postgres --test integration_postgres -- schema_and_accounts --nocapture`
- `cargo test -p sdkwork-api-interface-http --test canonical_account_admission -- --nocapture`

## Residual Risks

- downstream SDK/client regeneration from the updated public gateway schema has not yet been replayed in this loop

## Exit

- Step result: `conditional-go`
- Reason:
  - query, pagination, and index convergence is now closed on the public benefit-lot path
  - schema-consumer regeneration remains open
