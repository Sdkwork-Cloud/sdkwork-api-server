# 2026-04-10 Commercial S01 Offer Pricing Binding Review

## Scope

- Architecture reference: `docs/架构/166-API产品-商业账户-coupon-first营销统一架构-2026-04-10.md`
- Step reference: `docs/step/103-S01-API产品与定价主模型收敛-2026-04-10.md`
- Loop focus: bind canonical pricing semantics onto `ProductOffer` and align order pricing references to that offer truth

## Findings

### P0 - canonical offer truth still had no pricing owner binding

- `ApiProduct / ProductOffer / CatalogPublication` existed, but `PricingPlan / PricingRate` were still detached from catalog offers
- impact:
  - portal catalog could not answer which pricing plan/rate priced an offer
  - later market/admin/account slices would need to infer pricing from legacy buckets again

### P0 - order `pricing_plan_id` semantics were incorrect

- subscription order creation used `target_id` like `growth` as `pricing_plan_id`
- impact:
  - order pricing evidence did not reference the same pricing identity as catalog truth
  - pricing snapshot semantics were weaker than required for commercial auditability

## Fix Closure

- added additive pricing binding fields to `ProductOffer`
- generated deterministic canonical pricing ids and metric codes in `sdkwork-api-app-catalog`
- exposed pricing binding in portal catalog `offers`
- changed order creation to resolve `pricing_plan_id / pricing_plan_version` from canonical offer binding
- extended portal TS shared types and portal regression tests to prove the new contract

## Verification

- RED:
  - app-catalog offer serialization failed because `pricing_plan_id` was absent
  - portal catalog regression failed because offer pricing binding fields were absent
  - portal subscription checkout regression failed because `pricing_plan_id` returned raw `growth` instead of canonical pricing id
- GREEN:
  - `cargo test -p sdkwork-api-domain-catalog --test commercial_catalog -- --nocapture`
  - `cargo test -p sdkwork-api-app-catalog --test commercial_catalog -- --nocapture`
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_catalog_exposes_plans_packs_and_active_coupons -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_subscription_checkout_requires_settlement_before_membership_activation -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_order_detail_returns_canonical_order_view -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`
  - `node --input-type=module -` direct portal TS contract assertions

## Residual Risks

- quote contract still does not expose canonical pricing binding directly
- `pricing_snapshot_json` is still a local snapshot payload, not yet a canonical pricing-publication evidence document
- admin pricing lifecycle and canonical offer pricing binding are still parallel truths rather than a fully unified owner chain

## Exit

- Step result: `conditional-go`
- Reason:
  - `ProductOffer -> PricingPlan / PricingRate -> Order` linkage now exists
  - S01 still needs quote snapshot and pricing publication convergence before it is truly closed
