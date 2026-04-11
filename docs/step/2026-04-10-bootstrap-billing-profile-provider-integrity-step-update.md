# 2026-04-10 Bootstrap Billing Profile Provider Integrity Step Update

## What Changed

- Hardened bootstrap validation for `billing_events.applied_routing_profile_id`.
- A billing event that declares an applied routing profile now fails bootstrap unless its `provider_id` is actually declared by that routing profile.
- Declared providers are normalized as:
  - `ordered_provider_ids`
  - plus `default_provider_id` when it is not already present

## Why This Matters

- Before this step, bootstrap validated that a billing event's applied routing profile existed and belonged to the same workspace, but it did not validate that the billed provider was even part of that profile's route set.
- That left room for stale or contradictory billing evidence:
  - the event could claim it was routed under a profile
  - while billing a provider that the profile would never select
- In a commercial router this undermines auditability because billing lineage is supposed to explain why a provider was charged.

## Scope Boundary

- This step only validates the relationship:
  - `billing_events.provider_id` must belong to `billing_events.applied_routing_profile_id`
- It does **not** add a new requirement that every billing event must carry `applied_routing_profile_id`.
- It also does **not** replace the stricter snapshot-based provider check that already applies when `compiled_routing_snapshot_id` is present.

## Repository Audit

- Re-audited merged `prod` and `dev` profile packs across `routing/*.json`, `billing/*.json`, and all declared `updates/*.json`.
- Audit result for `prod`:
  - `BILLING_WITH_PROFILE=27`
  - `BILLING_PROVIDER_OUTSIDE_PROFILE=0`
- Audit result for `dev`:
  - `BILLING_WITH_PROFILE=30`
  - `BILLING_PROVIDER_OUTSIDE_PROFILE=0`

## Data Impact

- No repository `/data` seed files required changes.
- Existing `prod` and `dev` bootstrap packs already satisfy the new billing/profile/provider integrity rule.
- Idempotent bootstrap remains unchanged because this only rejects contradictory billing lineage.

## Test Coverage Added

- billing event rejects a provider that sits outside the declared provider set of its applied routing profile

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_billing_provider_outside_applied_routing_profile -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`
- Real `/data` audit:
  - `PROD BILLING_WITH_PROFILE=27 BILLING_PROVIDER_OUTSIDE_PROFILE=0`
  - `DEV BILLING_WITH_PROFILE=30 BILLING_PROVIDER_OUTSIDE_PROFILE=0`

## Follow-Up

- The next likely safe invariant is on billing-to-pricing lineage rather than more routing-profile tightening.
- A good candidate is to audit whether billing events that already have `provider_id + route_key + channel_id` can still drift from the effective commercial pricing posture that request metering and settlements later rely on.
