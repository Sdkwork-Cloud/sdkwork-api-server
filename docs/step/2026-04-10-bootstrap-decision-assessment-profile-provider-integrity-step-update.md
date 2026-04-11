# 2026-04-10 Bootstrap Decision Assessment Profile Provider Integrity Step Update

## What Changed

- Hardened bootstrap validation for `routing_decision_logs.assessments.provider_id`.
- When a routing decision log declares `applied_routing_profile_id`, every assessment provider must now belong to that routing profile's declared provider set.
- Declared providers are normalized as:
  - `ordered_provider_ids`
  - plus `default_provider_id` when it is not already present

## Why This Matters

- Before this step, bootstrap verified that:
  - the applied routing profile existed
  - the decision workspace matched that profile
  - the `selected_provider_id` belonged to that profile
- But assessment rows were still allowed to drift outside the profile whenever no compiled snapshot was attached.
- That weakened observability lineage:
  - the decision claimed a profile
  - the selected provider respected the profile
  - but the candidate set recorded in assessments could still show providers that profile would never evaluate
- For commercial routing analysis, this makes route simulation evidence less trustworthy.

## Scope Boundary

- This step validates only:
  - if `routing_decision_logs.applied_routing_profile_id` is present, every `routing_decision_logs.assessments.provider_id` must belong to that routing profile
- It does **not** require every decision log to carry `applied_routing_profile_id`.
- It does **not** change the stricter compiled snapshot path:
  - when `compiled_routing_snapshot_id` is present, assessment providers must still belong to the snapshot provider set as before

## Repository Audit

- Re-audited merged `prod` and `dev` profile packs across `routing/*.json`, `observability/*.json`, and all declared `updates/*.json`.
- Audit result for `prod`:
  - `DECISION_WITH_PROFILE=27`
  - `ASSESSMENT_OUTSIDE_PROFILE=0`
  - `ASSESSMENT_OUTSIDE_PROFILE_NO_SNAPSHOT=0`
- Audit result for `dev`:
  - `DECISION_WITH_PROFILE=30`
  - `ASSESSMENT_OUTSIDE_PROFILE=0`
  - `ASSESSMENT_OUTSIDE_PROFILE_NO_SNAPSHOT=0`

## Data Impact

- No repository `/data` seed files required changes.
- Existing `prod` and `dev` bootstrap packs already satisfy the new assessment/profile/provider integrity rule.
- Idempotent bootstrap behavior is unchanged because this only rejects contradictory observability evidence.

## Test Coverage Added

- routing decision log rejects an assessment provider outside the applied routing profile when no compiled snapshot is present

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_decision_assessment_provider_outside_applied_routing_profile -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`
- Real `/data` audit:
  - `PROD DECISION_WITH_PROFILE=27 ASSESSMENT_OUTSIDE_PROFILE=0 ASSESSMENT_OUTSIDE_PROFILE_NO_SNAPSHOT=0`
  - `DEV DECISION_WITH_PROFILE=30 ASSESSMENT_OUTSIDE_PROFILE=0 ASSESSMENT_OUTSIDE_PROFILE_NO_SNAPSHOT=0`

## Follow-Up

- The next safe tightening area remains pricing lineage.
- A strong candidate is parent-child pricing ownership consistency so `pricing_rates` cannot silently drift away from their parent `pricing_plans`.
