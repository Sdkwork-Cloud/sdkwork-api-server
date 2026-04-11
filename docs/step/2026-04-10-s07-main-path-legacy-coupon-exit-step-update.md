# 2026-04-10 S07 Main-Path Legacy Coupon Exit Step Update

## Scope

- Step: `S07 storage migration, dual-write gray release, and legacy exit`
- Loop target: remove legacy coupon fallback from the main runtime and primary admin coupon view
- Boundaries:
  - `sdkwork-api-interface-http`
  - `sdkwork-api-interface-portal`
  - `sdkwork-api-app-commerce`
  - `apps/sdkwork-router-admin`

## Changes

- Gateway / Portal coupon reads:
  - removed fallback from canonical code records to legacy `list_active_coupons()`
  - removed projected legacy coupon read path from public and portal main routes
- Commerce coupon reads and order-state handling:
  - removed legacy coupon merge from catalog assembly
  - removed compatibility coupon-context fallback during order coupon state transitions
  - removed reserve-time persistence of projected legacy template / campaign
- Main-path dependencies:
  - dropped legacy coupon app/domain compatibility dependencies from `sdkwork-api-app-commerce`
  - dropped legacy coupon app compatibility dependency from `sdkwork-api-interface-portal`
- Admin primary view:
  - removed legacy compatibility copy from the coupon detail panel
  - removed the matching marketing i18n entries and updated source-contract tests

## Verification

- RED:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-admin/tests/admin-legacy-coupon-exit.test.mjs`
    - failed because gateway / portal / commerce still referenced legacy fallback strings and admin still rendered legacy compatibility copy
- GREEN:
  - `node --test --experimental-test-isolation=none apps/sdkwork-router-admin/tests/admin-legacy-coupon-exit.test.mjs apps/sdkwork-router-admin/tests/admin-marketing-workbench.test.mjs apps/sdkwork-router-admin/tests/admin-i18n-coupons-governance-regressions.test.mjs`
  - `cargo check -p sdkwork-api-app-commerce -p sdkwork-api-interface-portal -p sdkwork-api-interface-http -j 1`

## Result

- gateway, portal, and commerce main coupon flows now resolve only from canonical marketing truth
- admin primary coupon detail no longer teaches operators a long-lived legacy compatibility mental model
- the old coupon system is no longer on the main runtime path, but deeper compatibility surfaces still exist

## Exit

- Step result: `conditional-go`
- Reason:
  - main-path legacy coupon fallback is removed
  - deeper legacy admin coupon compatibility APIs, storage/runtime cleanup, and cutover evidence remain open in later `S07` work
