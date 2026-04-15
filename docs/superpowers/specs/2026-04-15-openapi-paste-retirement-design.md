# OpenAPI Paste Retirement Design

**Date:** 2026-04-15

## Goal

Remove the remaining `RUSTSEC-2024-0436` informational advisory caused by `utoipa-axum -> paste` without destabilizing the admin and gateway OpenAPI surfaces that the product already verifies through route-level regression tests.

## Current Evidence

- the closure verification for this slice now reports `cargo audit --json --no-fetch --stale` with zero vulnerabilities and zero warnings
- `paste` and `utoipa-axum` no longer appear in the active workspace dependency graph
- `sdkwork-api-interface-admin` and `sdkwork-api-interface-http` now build their live OpenAPI documents through `#[derive(OpenApi)]` plus explicit `paths(...)`
- `sdkwork-api-interface-portal` no longer carries the stale OpenAPI UI dependency that was unrelated to its published contract surface
- the repository still retains route inventory regression tests for the admin and gateway OpenAPI JSON surfaces, which remain the verification oracle for OpenAPI drift

## Problem Statement

The hardening challenge in this slice was to retire the `paste` advisory without destabilizing business-critical admin and gateway OpenAPI contracts. A dependency cleanup that changed route registration semantics, schema discovery, or OpenAPI merging behavior would have created more risk than it removed, so the migration had to be verified against the existing route-level regression surfaces.

## Options Considered

### Option A: Replace `utoipa-axum` usage in application code

This would mean rewriting the admin and gateway OpenAPI assembly code to stop using `OpenApiRouter` and `routes!`.

Pros:

- removes the dependency at the application layer
- gives full internal control over OpenAPI assembly

Cons:

- large invasive change across two interface crates
- easy to introduce route or schema drift
- turns a dependency hygiene task into a behavior migration

### Option B: Vendor `utoipa-axum` and remove `paste` internally

This keeps the current `OpenApiRouter + routes!` call sites intact while moving the risk and change surface into a small vendored dependency patch.

Pros:

- minimal application-layer churn
- preserves the current OpenAPI builder API
- leverages existing admin and gateway regression tests
- keeps the supply-chain fix explicit and reviewable

Cons:

- introduces one more vendored crate to maintain
- requires careful macro patching inside the vendor

### Option C: Keep `paste` and allowlist it in policy

Pros:

- cheapest short-term change

Cons:

- does not actually retire the advisory
- leaves known governance debt in the shipping graph
- conflicts with the current hardening direction of the repository

## Recommendation

The implemented choice was Option A, because the workspace already contained an in-flight and compiling migration away from `utoipa-axum`.

During implementation planning, the lowest-risk path looked like vendoring `utoipa-axum`. After checking the real worktree, that is no longer the best choice:

- workspace manifests already removed `utoipa-axum`
- `sdkwork-api-interface-admin` and `sdkwork-api-interface-http` already migrated their OpenAPI assembly to `#[derive(OpenApi)]` with explicit `paths(...)`
- `cargo tree -i utoipa-axum --workspace` and `cargo tree -i paste --workspace` no longer find either crate in the active dependency graph

At this point, introducing a vendor would add churn back into a graph that has already been migrated off the dependency. The safer commercial path is to finish the direct removal cleanly: drop stale portal dependencies, regenerate `Cargo.lock`, and verify the OpenAPI surfaces still behave identically.

## Design

### Dependency Boundary

- Keep `utoipa-axum` out of the workspace manifests and source tree.
- Remove stale `utoipa-swagger-ui` from `sdkwork-api-interface-portal`, because the portal publishes a static OpenAPI document and does not host Swagger UI through that crate.
- Rewrite `Cargo.lock` from the current manifests so stale `utoipa-axum` and `paste` entries are pruned.

### Behavior Boundary

- Keep admin and gateway route definitions unchanged.
- Use explicit `#[derive(OpenApi)]` plus `paths(...)` as the OpenAPI source of truth for admin and gateway.
- Preserve the published JSON schema surfaces so the existing OpenAPI route tests remain the regression oracle.

### Verification Boundary

The implementation is only acceptable if all of the following remain true after the dependency change:

- `cargo audit` no longer reports `RUSTSEC-2024-0436`
- `Cargo.lock` no longer contains `name = "paste"`
- admin OpenAPI route regression tests still pass
- gateway OpenAPI route regression tests still pass
- `cargo check` remains green for the affected interface crates

## Risks And Mitigations

### Risk: explicit `paths(...)` lists drift from real route inventory

Mitigation:

- keep the explicit `paths(...)` lists aligned with the route regression tests
- rely on the repository's existing OpenAPI route assertions instead of assuming the manual lists are complete

### Risk: portal still pulls stale OpenAPI UI dependencies from its manifest

Mitigation:

- explicitly remove unused `utoipa-swagger-ui` from `crates/sdkwork-api-interface-portal/Cargo.toml`
- verify with `cargo tree -i utoipa-axum --workspace`

### Risk: the dependency disappears from audit but functional coverage regresses

Mitigation:

- keep OpenAPI route tests in the narrow TDD loop
- run final fresh verification with `cargo audit`, `cargo check`, and targeted test coverage

## Success Condition

This work is successful when the repository still publishes the same admin and gateway OpenAPI contract surfaces, `portal` no longer declares dead OpenAPI runtime dependencies, and the Rust dependency audit no longer reports `paste` in the active shipping graph.
