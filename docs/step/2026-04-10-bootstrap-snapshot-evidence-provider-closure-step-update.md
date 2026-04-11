# 2026-04-10 Bootstrap Snapshot Evidence Provider Closure Step Update

## Summary

This step closes another bootstrap data integrity gap in the routing evidence layer:

- `routing_decision_logs` could reference a compiled routing snapshot
- `billing_events` could reference a compiled routing snapshot
- but their provider fields still only needed to "exist", not belong to that snapshot's routed provider set

That allowed structurally valid but semantically broken evidence chains.

## What Changed

### 1. Added compiled snapshot provider-closure validation

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- when `routing_decision_logs.compiled_routing_snapshot_id` is set:
  - `selected_provider_id` must be declared by the referenced compiled routing snapshot
  - every `assessments[*].provider_id` must be declared by the referenced compiled routing snapshot
- when `billing_events.compiled_routing_snapshot_id` is set:
  - `provider_id` must be declared by the referenced compiled routing snapshot

This keeps bootstrap evidence aligned with how routing snapshots are produced at runtime:

- the snapshot is the canonical derived routing state
- decision logs and billing events must stay inside that state when they claim to reference it

### 2. Added regression coverage for provider leakage outside snapshots

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Added regression tests for:

- decision logs whose selected provider is outside the compiled snapshot provider set
- decision logs whose assessed providers are outside the compiled snapshot provider set
- billing events whose provider is outside the compiled snapshot provider set

### 3. Added billing provider-channel attribution validation

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`
- `crates/sdkwork-api-app-runtime/src/tests.rs`

New contract:

- when `billing_events.channel_id` is set, that channel must belong to the referenced `provider_id`

This prevents seeded billing evidence from mixing one provider with another channel's pricing surface.

Added regression test for:

- billing events whose `channel_id` is not bound to the referenced provider

### 4. Verified shipped `/data` still satisfies the stronger contract

No `/data` JSON change was required in this step.

The real bootstrap data already keeps decision/billing provider evidence inside the referenced snapshot route sets, so stricter validation did not break the default commercial seed packs.

## Verification

Passed:

- `cargo test -p sdkwork-api-app-runtime outside_snapshot -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime billing_channel_outside_provider_bindings -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Result

Bootstrap data now rejects a subtle but important class of evidence corruption:

- logs can no longer claim a compiled snapshot while selecting a provider outside that snapshot
- billing events can no longer claim a compiled snapshot while charging against a provider outside that snapshot
- billing events can no longer attribute usage to a channel that the provider is not actually bound to

This makes the seeded commercial routing evidence more trustworthy for admin diagnostics, billing inspection, and operational replay.
