# 2026-04-10 Bootstrap Decision Policy Integrity Step Update

## What Changed

- Hardened bootstrap validation for `routing_decision_logs.matched_policy_id`.
- A routing decision log now fails bootstrap unless its matched policy is:
  - present in `routing_policies`
  - `enabled = true`
  - actually matched by the decision log's own `capability + route_key`
- This closes the last obvious observability gap left after snapshot-policy integrity:
  - compiled snapshots are now validated against their matched policy
  - routing decision logs are now also validated against their matched policy

## Why This Matters

- Before this step, bootstrap could accept a routing decision log that structurally referenced a real policy id, but that policy could already be disabled or semantically unrelated to the logged route.
- That makes observability data lie about why a provider was selected.
- In a commercial router with pricing, routing, and audit trails, those stale decision-policy links are dangerous because admin, billing analysis, and route debugging all treat the matched policy as lineage evidence.

## Scope Boundary

- This step validates the decision log against the matched policy itself.
- It does **not** add a stronger new rule that every decision log must always carry `matched_policy_id`.
- It also does **not** add a new requirement beyond the existing snapshot consistency rule when `compiled_routing_snapshot_id` is present.
- The safe invariant here is:
  - if `routing_decision_logs.matched_policy_id` is present, it must point to an enabled policy that truly matches the decision's `capability + route_key`

## Repository Audit

- Re-audited merged `prod` and `dev` profile packs across `routing/*.json`, `observability/*.json`, and all declared `updates/*.json`.
- Audit result for `prod`:
  - `DECISION_MISSING_POLICY=0`
  - `DECISION_DISABLED_POLICY=0`
  - `DECISION_MISMATCHED_POLICY=0`
  - `DECISION_POLICY_SNAPSHOT_MISMATCH=0`
- Audit result for `dev`:
  - `DECISION_MISSING_POLICY=0`
  - `DECISION_DISABLED_POLICY=0`
  - `DECISION_MISMATCHED_POLICY=0`
  - `DECISION_POLICY_SNAPSHOT_MISMATCH=0`

## Data Impact

- No repository `/data` seed files required changes.
- Existing `prod` and `dev` bootstrap packs already satisfy the new decision-policy integrity rule.
- Idempotent bootstrap remains unchanged because this step only rejects contradictory or stale observability lineage.

## Test Coverage Added

- routing decision log rejects a disabled matched policy
- routing decision log rejects a matched policy whose `capability + model_pattern` does not match the decision's `capability + route_key`
- adjusted the existing `decision capability mismatched snapshot` regression so it still isolates snapshot mismatch instead of being intercepted by the new earlier decision-policy invariant

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_decision_with_disabled_matched_policy -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_decision_with_mismatched_matched_policy -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_decision_capability_mismatched_snapshot -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`
- Real `/data` audit:
  - `PROD DECISION_MISSING_POLICY=0 DECISION_DISABLED_POLICY=0 DECISION_MISMATCHED_POLICY=0 DECISION_POLICY_SNAPSHOT_MISMATCH=0`
  - `DEV DECISION_MISSING_POLICY=0 DECISION_DISABLED_POLICY=0 DECISION_MISMATCHED_POLICY=0 DECISION_POLICY_SNAPSHOT_MISMATCH=0`

## Follow-Up

- The next safe candidate invariant is likely on billing lineage, not more routing-policy strictness.
- A good next step is to audit whether `billing_events` that carry `compiled_routing_snapshot_id` or `provider_id + route_key` can still reference stale pricing or route-governance state that bootstrap does not yet reject.
