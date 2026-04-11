# 2026-04-10 Commercial S01 Quote Pricing Evidence Review

## Scope

- Architecture reference: `docs/架构/166-API产品-商业账户-coupon-first营销统一架构-2026-04-10.md`
- Step reference: `docs/step/103-S01-API产品与定价主模型收敛-2026-04-10.md`
- Loop focus: align quote payload and order pricing snapshot with canonical offer pricing truth

## Findings

### P0 - quote contract still lagged behind canonical offer truth

- catalog `offers` already exposed canonical pricing binding, but `PortalCommerceQuote` did not
- impact:
  - portal quote callers could not tie a quote to the same pricing identity as catalog truth
  - later account/marketing/order evidence would still need local inference

### P0 - order snapshot lacked explicit pricing evidence closure

- `pricing_snapshot_json` serialized request and quote data, but had no explicit canonical pricing-binding envelope
- impact:
  - order audit evidence preserved amounts but not a clear pricing-plan/rate/metric owner chain
  - settlement-side reconstruction risked drifting from quote-time truth

### P1 - custom recharge still needed canonical quote-level proof

- custom recharge order binding had already been corrected, but quote response had not yet proven the same special-case mapping
- impact:
  - `custom_recharge` could still look like a local pricing exception rather than a first-class canonical offer path

## Fix Closure

- extended `PortalCommerceQuote` with additive canonical pricing binding fields
- moved quote construction onto canonical offer pricing binding resolution
- made order creation persist pricing plan identity directly from quote truth
- added explicit `pricing_binding` envelope to `pricing_snapshot_json`
- rehydrated settlement quote pricing evidence from stored order snapshot/order record
- extended portal shared TS types and quote regressions to prove recharge-pack and custom-recharge quote parity

## Verification

- RED:
  - app-commerce quote-preview regression failed because quote/snapshot had no `pricing_plan_id`
  - portal quote regression failed because quote payload did not expose canonical pricing binding
- GREEN:
  - `cargo test -p sdkwork-api-app-commerce --test marketing_checkout_closure quote_preview -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_prices_recharge_and_coupon_redemption -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_quote_and_order_support_custom_recharge_from_server_policy -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_subscription_checkout_requires_settlement_before_membership_activation -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test portal_commerce portal_commerce_order_detail_returns_canonical_order_view -- --nocapture`
  - `cargo test -p sdkwork-api-interface-portal --test openapi_route openapi_routes_expose_portal_api_inventory_with_schema_components -- --nocapture`
  - `node --input-type=module -` direct portal TS contract assertions

## Residual Risks

- catalog publication evidence is still implicit; quote/order do not yet carry explicit publication ownership
- order storage still persists only `pricing_plan_id / pricing_plan_version` as first-class columns; rate/metric remain snapshot evidence
- admin-managed pricing lifecycle and current canonical catalog builder are still not one fully unified publication owner chain

## Exit

- Step result: `conditional-go`
- Reason:
  - `Offer -> Quote -> Order Snapshot` pricing evidence is now aligned
  - `Publication` evidence and stronger pricing-governance ownership are still pending before `S01` can be treated as truly closed
