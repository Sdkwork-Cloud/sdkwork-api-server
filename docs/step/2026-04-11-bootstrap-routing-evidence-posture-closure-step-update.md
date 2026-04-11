# 2026-04-11 Bootstrap Routing Evidence Posture Closure Step Update

## What Changed

- Hardened bootstrap validation for `compiled_routing_snapshots`, `routing_decision_logs`, and `provider_health_snapshots`.
- Added snapshot posture invariants:
  - `compiled_routing_snapshots.updated_at_ms` must not be earlier than `created_at_ms`
  - `compiled_routing_snapshots.strategy = deterministic_priority` requires `default_provider_id` to match the first ordered provider
- Added routing-decision evidence invariants:
  - every decision must carry exactly one assessment for `selected_provider_id`
  - the selected provider assessment must be `available = true`
  - when the linked compiled snapshot requires healthy routing, the selected provider assessment must be `health = healthy`
- Added provider-health runtime posture invariants:
  - `runtime = builtin` requires `instance_id`
  - `runtime = passthrough` must not declare `instance_id`
  - `healthy = true` requires both `running = true` and a non-empty `message`

## Why This Matters

- Bootstrap observability data is part of the commercial operating surface, not just sample decoration.
- Previous validation already covered:
  - workspace and profile/snapshot/provider ownership consistency
  - provider membership within snapshots and applied routing profiles
  - instance binding existence for provider health snapshots when an instance was declared
- That still left posture gaps:
  - a deterministic snapshot could advertise one default provider while ordering another provider first
  - a routing decision could claim one selected provider without carrying direct assessment evidence for that winner
  - a healthy runtime snapshot could still be non-running or omit operator-readable context
- In a commercial bootstrap pack, those gaps weaken trust in seeded route simulations, admin debugging, and runtime health evidence.

## Repository Audit

- Re-audited merged `prod` and `dev` bootstrap packs using profile + updates + last-wins collapse for:
  - `data/observability/*.json`
  - additive `data/updates/*.json`
- Audit result for the new closure set:
  - `PROFILE=prod DECISION_MISSING_SELECTED_ASSESSMENT=0 DECISION_SELECTED_ASSESSMENT_UNAVAILABLE=0 DECISION_SELECTED_ASSESSMENT_UNHEALTHY_WHEN_REQUIRED=0 DETERMINISTIC_SNAPSHOT_DEFAULT_MISMATCH=0 SNAPSHOT_UPDATED_BEFORE_CREATED=0`
  - `PROFILE=dev DECISION_MISSING_SELECTED_ASSESSMENT=0 DECISION_SELECTED_ASSESSMENT_UNAVAILABLE=0 DECISION_SELECTED_ASSESSMENT_UNHEALTHY_WHEN_REQUIRED=0 DETERMINISTIC_SNAPSHOT_DEFAULT_MISMATCH=0 SNAPSHOT_UPDATED_BEFORE_CREATED=0`
  - `PROFILE=prod NOT_RUNNING_BUT_HEALTHY=0 MISSING_INSTANCE_WHEN_BUILTIN=0 HAS_INSTANCE_WHEN_PASSTHROUGH=0 MISSING_MESSAGE_WHEN_HEALTHY=0`
  - `PROFILE=dev NOT_RUNNING_BUT_HEALTHY=0 MISSING_INSTANCE_WHEN_BUILTIN=0 HAS_INSTANCE_WHEN_PASSTHROUGH=0 MISSING_MESSAGE_WHEN_HEALTHY=0`
- Audit also checked `routing_decision_logs.created_at_ms >= compiled_routing_snapshots.updated_at_ms` and found drift:
  - `PROFILE=prod DECISION_BEFORE_SNAPSHOT_UPDATED=20`
  - `PROFILE=dev DECISION_BEFORE_SNAPSHOT_UPDATED=20`
- That temporal relation was intentionally not codified. The real data shows decision evidence and later snapshot refresh timestamps are not the same lifecycle event.

## Data Impact

- No `/data` seed files required changes.
- Existing repository bootstrap packs already satisfy the stronger routing evidence posture contract.
- One legacy runtime test fixture was normalized so it continues failing on its intended foreign-tenant account boundary instead of the new selected-assessment closure.

## Test Coverage Added

- builtin provider health snapshot rejects missing `instance_id`
- passthrough provider health snapshot rejects declared `instance_id`
- healthy provider health snapshot rejects `running = false`
- healthy provider health snapshot rejects missing `message`
- compiled snapshot rejects `updated_at_ms < created_at_ms`
- deterministic compiled snapshot rejects `default_provider_id` that does not match the first ordered provider
- routing decision rejects missing selected-provider assessment evidence
- routing decision rejects unavailable selected-provider assessment
- routing decision rejects non-healthy selected-provider evidence when the linked snapshot requires healthy routing

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- Future routing bootstrap packs should keep selected-provider evidence closed around the actual winner, not just candidate lists.
- If the product later needs separate concepts for runtime liveness and operator-facing health summaries, introduce explicit fields instead of weakening the current `healthy/running/message` posture contract.
