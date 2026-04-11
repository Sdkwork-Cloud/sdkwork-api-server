# 2026-04-10 Commercial S01 Catalog Publication Owner Chain Review

## Scope

- Architecture reference: `docs/架构/166-API产品-商业账户-coupon-first营销统一架构-2026-04-10.md`
- Step reference: `docs/step/103-S01-API产品与定价主模型收敛-2026-04-10.md`
- Loop focus: align portal catalog offer, quote, and order snapshot with canonical `ApiProduct / ProductOffer / CatalogPublication` owner chain

## Findings

### P0 - quote still lacked canonical product and offer identity

- quote had pricing binding, but still did not expose which canonical product/offer generated that quote
- impact:
  - quote consumers still needed local inference from `target_kind / target_id`
  - `ApiProduct / ProductOffer` layering remained incomplete at the transaction entry point

### P0 - publication evidence was only implicit in catalog builder internals

- portal catalog offers did not expose publication evidence, and order snapshot did not freeze it
- impact:
  - there was no explicit quote-time proof of which published offer surface the customer actually quoted against
  - later order-read canonicalization would need to infer publication identity from current catalog state

### P1 - snapshot compatibility path needed safe fallback

- older orders may not carry the new owner-chain fields once settlement-side quote rehydration starts relying on them
- impact:
  - adding publication evidence without fallback risked weakening compatibility for pre-upgrade snapshots

## Fix Closure

- extended quote/catalog binding from pricing-only to full `product / offer / publication / pricing`
- exposed publication evidence on portal catalog `offers`
- exposed canonical owner chain on portal quote payloads
- added explicit `catalog_binding` evidence into order snapshot
- restored owner chain from snapshot first during settlement quote rehydration, then fallback to live canonical catalog for compatibility
- extended portal shared TS contracts and portal regressions to prove the new owner chain

## Verification

- RED:
  - app-commerce quote-preview regression failed because quote/snapshot had no canonical `product_id`
  - portal quote regression failed because quote payload had no canonical product/offer/publication owner chain
  - portal catalog regression failed because catalog offers had no publication evidence
- GREEN:
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_plans_packs_and_active_coupons -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_prices_recharge_and_coupon_redemption -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_and_order_support_custom_recharge_from_server_policy -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_subscription_checkout_requires_settlement_before_membership_activation -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`
  - `node --input-type=module -` direct portal TS contract assertions

## Residual Risks

- outward order responses still do not expose canonical `product_id / offer_id / publication_*`
- publication ownership is still derived from builder defaults, not yet linked to admin-managed pricing/publication governance
- coupon-redemption remains intentionally outside canonical product/publication owner chain and still needs explicit long-term runtime boundary documentation

## Exit

- Step result: `conditional-go`
- Reason:
  - `Product -> Offer -> Publication -> Quote -> Order Snapshot` owner chain now exists
  - outward order contract and stronger publication governance ownership are still pending before `S01` can be considered fully closed
