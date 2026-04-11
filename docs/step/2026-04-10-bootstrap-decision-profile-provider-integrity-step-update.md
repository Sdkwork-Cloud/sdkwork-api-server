# 2026-04-10 Bootstrap Decision Profile Provider Integrity Step Update

## What Changed

- Hardened bootstrap validation for `routing_decision_logs.applied_routing_profile_id`.
- A routing decision log that declares an applied routing profile now fails bootstrap unless its `selected_provider_id` is actually declared by that routing profile.
- Declared providers are normalized as:
  - `ordered_provider_ids`
  - plus `default_provider_id` when it is not already present

## Why This Matters

- Before this step, bootstrap validated that a routing decision log's applied profile existed and belonged to the same workspace, but it did not validate that the selected provider was actually part of that profile's route set.
- That left a lineage gap:
  - the decision could claim it was made under a profile
  - while selecting a provider that profile would never allow
- In a commercial router, this weakens post-incident routing analysis and makes observability evidence less trustworthy.

## Scope Boundary

- This step validates only:
  - `routing_decision_logs.selected_provider_id` must belong to `routing_decision_logs.applied_routing_profile_id`
- It does **not** add a rule that all assessment providers must belong to the applied profile.
- It also does **not** require every decision log to carry `applied_routing_profile_id`.
- Snapshot-based provider validation remains a separate, stricter path when `compiled_routing_snapshot_id` is present.

## Repository Audit

- Re-audited merged `prod` and `dev` profile packs across `routing/*.json`, `observability/*.json`, and all declared `updates/*.json`.
- Audit result for `prod`:
  - `DECISION_WITH_PROFILE=27`
  - `DECISION_PROVIDER_OUTSIDE_PROFILE=0`
- Audit result for `dev`:
  - `DECISION_WITH_PROFILE=30`
  - `DECISION_PROVIDER_OUTSIDE_PROFILE=0`

## Data Impact

- No repository `/data` seed files required changes.
- Existing `prod` and `dev` bootstrap packs already satisfy the new decision/profile/provider integrity rule.
- Idempotent bootstrap behavior is unchanged because this only rejects contradictory routing lineage.

## Test Coverage Added

- routing decision log rejects a selected provider that sits outside the declared provider set of its applied routing profile

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_decision_provider_outside_applied_routing_profile -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`
- Real `/data` audit:
  - `PROD DECISION_WITH_PROFILE=27 DECISION_PROVIDER_OUTSIDE_PROFILE=0`
  - `DEV DECISION_WITH_PROFILE=30 DECISION_PROVIDER_OUTSIDE_PROFILE=0`

## Follow-Up

- If routing observability tightening continues, the next candidate is assessment-level governance:
  - whether every `routing_decision_logs.assessments.provider_id` should also be constrained to the applied profile when no compiled snapshot is attached
- That should be audited separately because it is a stronger rule than the selected-provider invariant added here.
