# 2026-04-10 Bootstrap Commerce Order Coupon Code Canonical Closure Step Update

## What Changed

- Restored canonical bootstrap coverage for `commerce_orders.applied_coupon_code`.
- Updated bootstrap fixture and repository seed data so every seeded order coupon code resolves through canonical `coupon_codes.code_value`.
- Added regression assertions in runtime bootstrap tests for:
  - local fixture `LAUNCH100`
  - repository `prod` profile `LAUNCH100`
  - repository `dev` profile `LAUNCH100`, `DEV50`, `PARTNER30`, and `GLABAPAC3000`

## Why This Matters

- `commerce_orders.applied_coupon_code` is not decorative text in this system.
- Downstream admin audit and commerce drill-down attempt to resolve that value back to a canonical coupon code record when reservation or redemption records are absent.
- That means bootstrap data must preserve a real canonical relationship:
  - order applied code
  - coupon code record
  - coupon template
  - marketing campaign
- The regression surfaced because some seed files still carried legacy or alias coupon strings in `marketing.coupons`, while bootstrap and runtime now trust canonical `coupon_codes`.
- Without this fix, bootstrap failed early and historical order audit would lose coupon lineage even if the marketing template still existed.

## Repository Audit

- Re-audited profile-visible order coupon codes against canonical marketing code values for:
  - `data/commerce/default.json`
  - `data/commerce/dev.json`
  - update packs referenced by `data/profiles/prod.json`
  - update packs referenced by `data/profiles/dev.json`
  - `data/marketing/default.json`
  - `data/marketing/dev.json`
  - marketing update packs that publish canonical coupon codes
- Audit result after the fix:
  - `PROFILE=prod MISSING=`
  - `PROFILE=dev MISSING=`
- Effective profile-visible order coupon coverage now includes:
  - `prod`: `LAUNCH100`, `PROVIDERMIX15`, `CNDIRECT888`, `OFFICIALDIRECT100`, `EDGELOCAL20`, `ENTERPRISE500`
  - `dev`: `LAUNCH100`, `DEV50`, `PARTNER30`, `GLABAPAC3000`, `PROVIDERMIX15`, `CNDIRECT888`, `OFFICIALDIRECT100`, `EDGELOCAL20`, `ENTERPRISE500`

## Data Impact

- Updated bootstrap fixture marketing data in [tests.rs](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/crates/sdkwork-api-app-runtime/src/tests.rs) so the local seed order coupon `LAUNCH100` has a canonical coupon code record.
- Updated repository marketing seed data:
  - [default.json](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/marketing/default.json)
  - [dev.json](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/data/marketing/dev.json)
- Specific fixes:
  - added canonical `LAUNCH100`
  - normalized shared-code dev values from sandbox aliases to `DEV50` and `PARTNER30`
  - normalized default shared partner code to `PARTNER20`

## Test Coverage Added

- local bootstrap fixture load now asserts canonical `LAUNCH100` coverage
- repository `prod` bootstrap discovery now asserts canonical `LAUNCH100` coverage
- repository `dev` bootstrap discovery now asserts canonical coverage for default, dev, and growth-lab order coupon codes
- product runtime bootstrap tests now assert the same canonical coupon code availability

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_loads_bootstrap_profile_data_pack -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_discovers_repository_bootstrap_profile_data_pack -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_discovers_repository_dev_bootstrap_profile_data_pack -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- `marketing.coupons` is still legacy compatibility-shaped seed content; canonical bootstrap correctness now depends on `coupon_codes`.
- Future shared-code seed updates must keep public coupon strings and canonical `coupon_codes.code_value` aligned, or the legacy `coupons` block should be retired entirely from bootstrap sources.
- If future profiles introduce more alias-style coupon presentation, keep aliases outside transactional bootstrap facts and keep `commerce_orders.applied_coupon_code` pinned to canonical code values.
