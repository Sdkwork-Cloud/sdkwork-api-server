# 2026-04-10 Commercial S07 Main-Path Legacy Coupon Exit Review

> Status: historical review snapshot. The remaining risks recorded here were fully closed later on 2026-04-10 by the full legacy coupon exit.

## Scope

- Architecture reference: `166`
- Step reference: `109`
- Loop focus: remove legacy coupon compatibility from main runtime paths before deeper cutover

## Findings

### P0 - gateway / portal / commerce coupon flows still depended on legacy coupon fallback

- main-path loaders still fell back to legacy coupon list and projection helpers
- reserve flows still treated projected legacy template / campaign as writable runtime truth
- impact:
  - canonical marketing was not the sole runtime coupon fact source
  - new coupon behavior could still be masked by compatibility-era reads

### P1 - admin primary coupon detail still surfaced long-lived legacy compatibility copy

- the main coupon detail panel still rendered legacy compatibility copy
- impact:
  - operator mental model remained split between canonical marketing governance and compatibility-era coupon posture

## Fix Closure

- removed legacy fallback reads from gateway, portal, and commerce main coupon context resolution
- removed reserve-time persistence of projected legacy template / campaign
- dropped main-path legacy coupon crate dependencies from commerce and portal
- removed legacy compatibility copy from the admin primary coupon detail view and i18n surface

## Verification

- `node --test --experimental-test-isolation=none apps/sdkwork-router-admin/tests/admin-legacy-coupon-exit.test.mjs apps/sdkwork-router-admin/tests/admin-marketing-workbench.test.mjs apps/sdkwork-router-admin/tests/admin-i18n-coupons-governance-regressions.test.mjs`
- `cargo check -p sdkwork-api-app-commerce -p sdkwork-api-interface-portal -p sdkwork-api-interface-http -j 1`

## Residual Risks

- historical at review time: admin still exposed legacy coupon compatibility routes
- storage/runtime-level legacy coupon crates and migration deletion are not removed by this slice

## Exit

- Step result: `conditional-go`
- Reason:
  - main-path runtime and primary admin view are cleaned
  - deep compatibility APIs and full legacy dependency deletion remain open
