# 2026-04-10 Bootstrap Routing Evidence Provider Account Closure Step Update

## What Changed

- Hardened bootstrap validation so every provider declared by `compiled_routing_snapshots` must be a tenant-accessible executable provider for the snapshot workspace.
- Hardened bootstrap validation so `routing_decision_logs.selected_provider_id` must be a tenant-accessible executable provider for the decision workspace.
- Hardened bootstrap validation so every `routing_decision_logs.assessments[*].provider_id` must be a tenant-accessible executable provider for the decision workspace.
- Added regression tests for:
  - compiled routing snapshots that include a provider backed only by another tenant's account
  - routing decisions whose selected provider is backed only by another tenant's account
  - routing decisions whose assessed provider is backed only by another tenant's account

## Why This Matters

- A routed provider appearing in observability evidence is not commercially valid just because the provider exists.
- The provider must also be executable for that tenant workspace; otherwise the snapshot or decision cannot be replayed, diagnosed, or trusted as real route evidence.
- Without this closure, bootstrap could accept observability records that point at providers available only to another tenant, which breaks admin diagnostics and makes seeded commercial workspaces less trustworthy.

## Data Impact

- No repository `/data` seed pack required changes.
- Repository audits confirmed:
  - every `compiled_routing_snapshots` provider in `/data/observability/*.json` is already tenant-accessible and executable
  - every `routing_decision_logs.selected_provider_id` and `assessments[*].provider_id` in `/data/observability/*.json` is already tenant-accessible and executable
- Two legacy synthetic tests needed fixture cleanup so they would keep failing for their intended billing/job provider-account reason instead of tripping over the stronger observability closure first.

## Verification

- `cargo test -p sdkwork-api-app-runtime compiled_snapshot_provider_with_only_foreign_tenant_account -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime decision_selected_provider_with_only_foreign_tenant_account -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime decision_assessment_provider_with_only_foreign_tenant_account -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime bootstrap -- --nocapture`
- `cargo test -p sdkwork-api-product-runtime bootstrap -- --nocapture`

## Follow-Up

- The next worthwhile closure candidate is route-price readiness in observability fixtures:
  - make synthetic snapshot and decision fixtures route-realistic per provider candidate
  - then consider hard validation that snapshot candidate sets also have active price coverage for the referenced route
