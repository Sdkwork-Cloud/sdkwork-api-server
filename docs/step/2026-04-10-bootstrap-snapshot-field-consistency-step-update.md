# 2026-04-10 Bootstrap Snapshot Field Consistency Step Update

## Summary

This step hardens bootstrap evidence integrity beyond provider-set closure:

- `routing_decision_logs` could already reference a compiled routing snapshot
- `billing_events` could already reference a compiled routing snapshot
- but they still did not need to match the snapshot's own routed model semantics

That left room for a subtle but serious failure mode:

- the evidence chain could point at a real snapshot
- the provider could belong to that snapshot
- yet the record could still describe a different `capability`, `route_key`, `strategy`, or applied routing profile

For seeded commercial data, that is still semantically broken.

## What Changed

### 1. Added decision-log to snapshot field consistency validation

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- when `routing_decision_logs.compiled_routing_snapshot_id` is set, the decision log must match the referenced snapshot for:
  - `capability`
  - `route_key`
  - `strategy`
  - `matched_policy_id`
  - `applied_routing_profile_id`

This treats the compiled snapshot as canonical routing evidence, not just a loose foreign key.

### 2. Added billing-event to snapshot field consistency validation

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- when `billing_events.compiled_routing_snapshot_id` is set, the billing event must match the referenced snapshot for:
  - `capability`
  - `route_key`
  - `applied_routing_profile_id`

This keeps billed usage explainable by the same routed model context that the snapshot claims.

### 3. Added regression coverage for snapshot semantic drift

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Added regression tests for:

- decision logs whose `capability` drifts from the referenced snapshot
- decision logs whose `route_key` drifts from the referenced snapshot
- decision logs whose `strategy` drifts from the referenced snapshot
- decision logs whose `matched_policy_id` drifts from the referenced snapshot
- billing events whose `capability` drifts from the referenced snapshot
- billing events whose `route_key` drifts from the referenced snapshot
- billing events whose `applied_routing_profile_id` drifts from the referenced snapshot

### 4. Repaired shipped `/data` evidence to satisfy the stronger contract

Updated:

- `data/observability/2026-04-global-provider-operations-readiness.json`
- `data/billing/2026-04-global-pricing-posture-operations-linkage.json`

Discovered issue during repository bootstrap verification:

- `billing-prod-openai-official-direct-2026` described `provider-openai-official` usage for `gpt-4.1-mini`
- but it referenced `snapshot-prod-openai-official-live`, whose canonical `route_key` is `gpt-4.1`

Fix applied:

- added `snapshot-prod-openai-official-direct-2026`
- added matching `decision-prod-openai-official-direct-2026`
- retargeted `billing-prod-openai-official-direct-2026` to the new snapshot

This preserves a complete commercial evidence chain instead of weakening validation.

## Verification

Passed:

- `cargo test -p sdkwork-api-app-runtime mismatched_snapshot -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime billing_ -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_loads_bootstrap_profile_data_pack -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Result

Bootstrap data is now stricter in the right place:

- compiled snapshots behave as canonical routing evidence
- logs and billing records cannot silently drift away from the routed model they claim to explain
- shipped `/data` now includes the missing OpenAI official mini-route snapshot chain needed for the stronger contract

This improves seeded commercial readiness for admin diagnostics, billing attribution, route replay, and future `/data` update packs.
