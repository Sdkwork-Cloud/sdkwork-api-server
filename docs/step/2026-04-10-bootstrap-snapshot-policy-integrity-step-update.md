# 2026-04-10 Bootstrap Snapshot Policy Integrity Step Update

## What Changed

- Hardened bootstrap validation for `compiled_routing_snapshots.matched_policy_id`.
- A compiled routing snapshot now fails bootstrap unless its matched policy is:
  - present in `routing_policies`
  - `enabled = true`
  - actually matched by the snapshot's own `capability + route_key`
- The new check runs after reference validation and before downstream provider/default-provider consistency checks.

## Why This Matters

- Before this change, bootstrap could accept a compiled routing snapshot that claimed to be derived from a policy that was disabled or no longer matched the route key.
- That creates stale observability seeds: admin and billing can see a historical routing decision shape that no longer reflects a valid route-governance source of truth.
- The new invariant closes that gap at bootstrap time and keeps snapshot lineage aligned with real route policy semantics.

## Scope Boundary

- This step intentionally enforces snapshot-to-policy integrity only.
- It does **not** enforce the stronger rule that every compiled snapshot provider list must be a full structural projection of the matched policy.
- That stronger rule is riskier because current repository seeds still rely on broader fallback composition in some places; the safer invariant is:
  - if a snapshot declares `matched_policy_id`, that policy must exist, be enabled, and truly match the snapshot

## Repository Audit

- Re-audited the real `/data` profile packs by loading `prod` and `dev` profile manifests plus all declared updates and then checking merged `routing` + `observability` payloads.
- Audit result for `prod`:
  - `SNAPSHOT_MISSING_POLICY=0`
  - `SNAPSHOT_DISABLED_POLICY=0`
  - `SNAPSHOT_MISMATCHED_POLICY=0`
  - `SNAPSHOT_INACTIVE_PROFILE=0`
- Audit result for `dev`:
  - `SNAPSHOT_MISSING_POLICY=0`
  - `SNAPSHOT_DISABLED_POLICY=0`
  - `SNAPSHOT_MISMATCHED_POLICY=0`
  - `SNAPSHOT_INACTIVE_PROFILE=0`

## Data Impact

- No repository `/data` seed files required changes for this invariant.
- Existing `prod` and `dev` bootstrap packs already satisfy the new snapshot-policy integrity rule.
- Idempotent bootstrap behavior is preserved because this step only rejects stale or contradictory snapshot metadata.

## Test Coverage Added

- compiled routing snapshot rejects a disabled matched policy
- compiled routing snapshot rejects a matched policy whose `capability + model_pattern` does not match the snapshot's `capability + route_key`

## Verification

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_compiled_snapshot_with_disabled_matched_policy -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_bootstrap_compiled_snapshot_with_mismatched_matched_policy -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- Real `/data` audit:
  - `PROD SNAPSHOT_MISSING_POLICY=0 SNAPSHOT_DISABLED_POLICY=0 SNAPSHOT_MISMATCHED_POLICY=0 SNAPSHOT_INACTIVE_PROFILE=0`
  - `DEV SNAPSHOT_MISSING_POLICY=0 SNAPSHOT_DISABLED_POLICY=0 SNAPSHOT_MISMATCHED_POLICY=0 SNAPSHOT_INACTIVE_PROFILE=0`

## Follow-Up Verification Status

- A subsequent workspace verification run completed after the admin marketing lifecycle integration was brought back into a compilable state.
- Follow-up result:
  - `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture` passed
- That means the snapshot-policy integrity step now sits on a fully green bootstrap verification chain across both:
  - `sdkwork-api-app-runtime`
  - `sdkwork-api-product-runtime`
