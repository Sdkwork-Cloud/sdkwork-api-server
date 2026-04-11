# 2026-04-10 Bootstrap Routing Workspace Consistency Step Update

## Summary

This step closes a workspace-consistency gap in bootstrap routing data.

Two route-facing references were still only checked for existence:

- `project_preferences.preset_id`
- `api_key_groups.default_routing_profile_id`

That allowed bootstrap data to stay referentially valid while still binding one workspace to another workspace's routing profile.

## What Changed

### 1. Hardened project preference preset ownership

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- `project_preferences.preset_id` must reference a routing profile in the same workspace as `project_preferences.project_id`
- the referenced profile must match both:
  - the owning tenant of the project
  - the same project id

This prevents project-level default routing posture from pointing at another tenant or another project.

### 2. Hardened API key group default routing profile ownership

Updated:

- `crates/sdkwork-api-app-runtime/src/bootstrap_data/manifest.rs`

New contract:

- `api_key_groups.default_routing_profile_id` must reference a routing profile in the same `(tenant_id, project_id)` workspace as the API key group

This keeps traffic-group defaults aligned with the workspace that will actually issue and meter requests through that group.

### 3. Added regression coverage for cross-workspace routing references

Updated:

- `crates/sdkwork-api-app-runtime/src/tests.rs`

Added regression tests for:

- project preferences referencing a routing profile from another workspace
- API key groups referencing a default routing profile from another workspace

## Verification

Passed:

- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_project_preferences_preset_from_other_workspace -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_rejects_api_key_group_default_routing_profile_from_other_workspace -- --nocapture`
- `cargo test -p sdkwork-api-app-runtime build_admin_store_from_config_loads_bootstrap_profile_data_pack -- --nocapture`

## Result

Bootstrap routing relationships are now more coherent:

- route profile references are no longer just existent
- they must also belong to the correct workspace

That reduces the chance of shipping `/data` packs that look structurally complete but leak routing posture across tenant or project boundaries.
