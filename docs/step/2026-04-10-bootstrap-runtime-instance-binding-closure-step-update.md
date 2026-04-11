# 2026-04-10 Bootstrap Runtime Instance Binding Closure Step Update

## What Changed

- Hardened bootstrap validation so every `extension_runtime_rollouts` record with `scope = "instance"` must target an instance that is backed by at least one executable provider-account binding.
- Hardened bootstrap validation so every `provider_health_snapshots.provider_id + instance_id` pair must map to an executable provider-account binding.
- Added regression tests for:
  - instance-scoped runtime rollouts that target a real extension instance but no executable provider account binds it
  - provider health snapshots that reference a real instance but no executable provider account binds it

## Why This Matters

- Runtime governance and health evidence should only describe execution paths the system can actually use.
- Without this closure, bootstrap could accept:
  - rollout records targeting instances that are installed but commercially unroutable
  - health snapshots for instances that have no executable provider-account path into real traffic
- That creates noisy operations data and weakens trust in seeded admin observability.

## Data Impact

- No repository `/data` seed pack required changes.
- Repository audits confirmed:
  - all `provider_health_snapshots` with `instance_id` already map to executable provider-account bindings
  - all `instance` scoped extension runtime rollouts already target executable provider-account bindings
- One existing synthetic rollout-participant regression test needed its profile to load `provider_accounts/default.json` so it would keep failing for the intended missing-node reason under the stronger rollout binding contract.

## Verification

- `cargo test -p sdkwork-api-app-runtime extension_runtime_rollout_instance_without_executable_provider_account_binding -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime provider_health_snapshot_without_executable_provider_account_binding -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime extension_runtime_rollout_participant_with_unknown_node -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- The next strong candidate is rollout scope commercial readiness:
  - for `scope = "extension"` or `scope = "instance"`, bind rollout targets to actual catalog/provider ownership where possible
  - ensure seeded runtime governance data never references extension targets that are operationally installed but commercially detached from provider routing
