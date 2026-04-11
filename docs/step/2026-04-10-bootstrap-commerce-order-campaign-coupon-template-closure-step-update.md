# 2026-04-10 Bootstrap Commerce Order Campaign Coupon Template Closure Step Update

## What Changed

- Hardened bootstrap validation for `commerce_orders` when both of these fields are present:
  - `marketing_campaign_id`
  - `applied_coupon_code`
- Bootstrap now requires the resolved canonical coupon code record and the linked marketing campaign to point at the same `coupon_template_id`.

## Why This Matters

- Previous validation already ensured:
  - `marketing_campaign_id` exists
  - `applied_coupon_code` exists in canonical `coupon_codes`
- That still left a lineage gap:
  - an order could point to a real campaign
  - point to a real coupon code
  - but those two records could belong to different coupon templates and still pass bootstrap
- In commercial data, that would corrupt coupon attribution, campaign subsidy analysis, and admin order audit, because the order would no longer have one coherent marketing source.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/marketing/*.json`
  - `data/commerce/*.json`
- Audit result:
  - `PROFILE=prod TEMPLATE_MISMATCH=0`
  - `PROFILE=prod MATRIX={"template-launch-credit-100::template-launch-credit-100":1,"template-provider-mix-15::template-provider-mix-15":1,"template-china-direct-888::template-china-direct-888":1,"template-official-direct-100::template-official-direct-100":1,"template-edge-local-20::template-edge-local-20":1,"template-enterprise-contract-500::template-enterprise-contract-500":1}`
  - `PROFILE=dev TEMPLATE_MISMATCH=0`
  - `PROFILE=dev MATRIX={"template-launch-credit-100::template-launch-credit-100":1,"template-dev-sandbox-50::template-dev-sandbox-50":1,"template-partner-sandbox-30::template-partner-sandbox-30":1,"template-provider-mix-15::template-provider-mix-15":1,"template-china-direct-888::template-china-direct-888":1,"template-official-direct-100::template-official-direct-100":1,"template-edge-local-20::template-edge-local-20":1,"template-enterprise-contract-500::template-enterprise-contract-500":1,"template-growth-lab-apac-30::template-growth-lab-apac-30":1}`
- Current merged data already treats order campaign lineage and order coupon lineage as one template-level fact.

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger order-to-marketing lineage contract.
- A focused regression fixture was added by mutating the local bootstrap fixture so an order kept `LAUNCH100` but was reassigned to a different campaign template; bootstrap now rejects that drift explicitly.

## Test Coverage Added

- order rejects applied coupon code that resolves to a different template than the linked marketing campaign

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_commerce_order_with_coupon_code_outside_marketing_campaign_template -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- If future bootstrap packs start seeding coupon reservations or redemptions directly, extend the same closure rule so:
  - reservation
  - redemption
  - applied coupon code
  - marketing campaign
  all collapse onto the same canonical coupon template lineage.
- Until those states are materially exercised in repository seed data, keeping this invariant scoped to currently used order fields preserves rigor without overfitting.
