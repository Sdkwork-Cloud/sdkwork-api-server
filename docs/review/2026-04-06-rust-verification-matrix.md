# 2026-04-06 Rust Verification Matrix

## Purpose

This document turns the current Rust verification gap into an explicit package-level execution matrix that can be repeated locally and in CI without relying only on one large `cargo check` invocation.

The current evidence base is:

- targeted `cargo test` coverage already exists for the request-path and OpenAPI surfaces touched in this review round
- historical long-running multi-package checks surfaced native dependency hotspots around `libz-ng-sys`
- the Windows default workspace target path also reproduced `link.exe` `LNK1201` failures while proc-macro crates tried to write `.pdb` files under the long shared `target` tree
- forcing `RUSTFLAGS='-C debuginfo=0'` is not a safe blanket workaround on this workstation because Rust `1.92.0` can ICE in transitive crates such as `zlib-rs` and `windows-sys`
- a managed short `CARGO_TARGET_DIR` plus the repository-local `zip` patch now allows `cargo check --workspace -j 1` to complete successfully on this Windows workstation

This matrix is designed to keep confidence high while reducing timeout risk.

## Automation

The matrix is now executable through repository-owned automation instead of remaining a docs-only checklist:

- orchestration script:
  - `scripts/check-rust-verification-matrix.mjs`
- script regression coverage:
  - `scripts/check-rust-verification-matrix.test.mjs`
- workflow regression coverage:
  - `scripts/rust-verification-workflow.test.mjs`
- CI workflow:
  - `.github/workflows/rust-verification.yml`

The workflow fans out the same split-package groups documented below on `ubuntu-latest`:

- `interface-openapi`
- `gateway-service`
- `admin-service`
- `portal-service`
- `product-runtime`

The script now also exposes an optional local deep-validation group:

- `workspace`

The workflow also exposes a manual hosted deep-validation lane:

- `workflow_dispatch` with `group=workspace`
  - runs `node scripts/check-rust-verification-matrix.mjs --group workspace`
  - executes on `windows-latest`
  - is intentionally separated from the default PR matrix so regular pull requests keep the faster split-package cadence

The script standardizes:

- a managed short `CARGO_TARGET_DIR` on Windows through `scripts/workspace-target-dir.mjs`
- a normal workspace `target` directory on non-Windows hosts unless a caller overrides `CARGO_TARGET_DIR`
- Windows `rustup.exe` fallback handling for shells where `%USERPROFILE%` or the local cargo bin path is unavailable
- Windows CMake generator normalization for `libz-ng-sys`

The script intentionally does **not** inject `RUSTFLAGS='-C debuginfo=0'` anymore because that flag combination caused compiler ICEs in this environment during full-workspace verification.

## Execution Rules

1. Reuse a dedicated short target directory on Windows so native dependencies and proc-macro outputs do not rebuild into an over-deep shared path.
2. Prefer `-j 1` for the package-level `cargo check` gates when native dependencies are cold or the machine is resource-constrained.
3. Run only the gates that correspond to the packages or behavior changed in the current slice.
4. Record exact passing commands back into `docs/review/2026-04-06-application-review.md`.
5. Treat timeouts as verification gaps unless a concrete compile error is observed.

## Environment Setup

### PowerShell

```powershell
Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue
node scripts/check-rust-verification-matrix.mjs --group gateway-service
```

### Bash

```bash
unset RUSTFLAGS
node scripts/check-rust-verification-matrix.mjs --group gateway-service
```

If you need a fresh isolated cache for a one-off local run, override `CARGO_TARGET_DIR` explicitly with a short path, for example `t4` inside the workspace, and keep `RUSTFLAGS` unset.

## Minimal Required Gates

| Change surface | Required commands | Why |
| --- | --- | --- |
| `sdkwork-api-interface-http` request-path behavior | `cargo test -j 1 -p sdkwork-api-interface-http --test <affected_suite>` | Proves the exact HTTP fallback or error-mapping behavior that changed. |
| gateway/admin/portal OpenAPI or router exposure surface | `cargo test -j 1 -p sdkwork-api-interface-http --test openapi_route` | Confirms the public gateway router still builds and exposes OpenAPI safely. |
| gateway/admin/portal OpenAPI or router exposure surface | `cargo test -j 1 -p sdkwork-api-interface-admin --test openapi_route` | Confirms the admin interface router still builds and exposes OpenAPI safely. |
| gateway/admin/portal OpenAPI or router exposure surface | `cargo test -j 1 -p sdkwork-api-interface-portal --test openapi_route` | Confirms the portal interface router still builds and exposes OpenAPI safely. |
| gateway service bootstrap/runtime wiring | `cargo check -j 1 -p gateway-service` | `gateway-service` depends on `sdkwork-api-interface-http` plus runtime/config/storage integration and is the first binary-level gate for gateway startup. |
| admin service bootstrap/runtime wiring | `cargo check -j 1 -p admin-api-service` | `admin-api-service` is the binary-level gate for the admin interface dependency chain. |
| portal service bootstrap/runtime wiring | `cargo check -j 1 -p portal-api-service` | `portal-api-service` is the binary-level gate for the portal interface dependency chain. |
| product runtime library changes | `cargo check -j 1 -p sdkwork-api-product-runtime` | Confirms the shared product runtime still compiles after interface/runtime integration changes. |
| product runtime binary entrypoint changes | `cargo check -j 1 -p router-product-service` | Verifies the service wrapper over `sdkwork-api-product-runtime` still compiles. |
| Windows full-workspace deep verification | `node scripts/check-rust-verification-matrix.mjs --group workspace` | Reuses the managed short target directory and proves the entire Rust workspace builds under the repository-owned verification entrypoint. |

## Verified Command Families Already Available

The current review round already proved the following command families, and they should be reused before expanding the matrix:

- targeted `sdkwork-api-interface-http` route suites for not-found and invalid-request handling
- exact regression tests for request-path panic removals
- gateway/admin/portal `openapi_route` tests

The authoritative command log for those passing runs remains:

- `docs/review/2026-04-06-application-review.md`

## Recommended Order

Run the smallest meaningful gate first:

1. affected `cargo test` suite for the changed route or handler
2. relevant interface `openapi_route` tests if router construction or exposure changed
3. package-level `cargo check -j 1` for each touched service or runtime package
4. only if all package-level gates are green, consider a broader multi-package `cargo check`

This order preserves the fastest feedback loop and avoids spending native compile time before the behavior-level regressions are proven green.

## Native Dependency Hotspots

### `libz-ng-sys`

Observed behavior in this workspace:

- the monolithic multi-package `cargo check` timed out while native dependencies were compiling
- `libz-ng-sys` was the most visible hotspot in the timeout output

Operational guidance:

1. Warm the shared target directory before large verification runs.
2. Split service and runtime checks into separate commands.
3. Keep `-j 1` for cold-cache runs on CI or developer workstations with limited CPU or memory headroom.
4. Only re-run the package that actually changed after a failure; do not restart the entire service set immediately.

### `zip` / vendored Swagger UI extraction

Observed behavior in this workspace:

- `utoipa-swagger-ui` pulls `zip 3.0.0` as a build dependency
- upstream `zip 3.0.0` maps its `deflate` feature to `flate2/zlib-rs`
- on Rust `1.92.0` for `x86_64-pc-windows-msvc`, that path reproduced compiler ICEs during full-workspace verification

Operational guidance:

1. Keep the repository-local `zip` patch in place.
2. Route `zip` deflate operations through `flate2/rust_backend` instead of `zlib-rs`.
3. Treat any reintroduction of `zlib-rs` into the workspace verification path as a release-blocking regression for Windows verification.

## Current Status

Completed in this document:

- minimal required verification matrix defined
- long-running service/runtime verification split into smaller package gates
- native dependency hotspot guidance documented for `libz-ng-sys`
- vendored Swagger UI `zip` dependency patched away from `zlib-rs`
- automation script added at `scripts/check-rust-verification-matrix.mjs`
- CI workflow added at `.github/workflows/rust-verification.yml`
- automation regression tests added for the script and workflow wiring
- all documented package groups completed successfully in local split-package runs:
  - `node scripts/check-rust-verification-matrix.mjs --group interface-openapi`
  - `node scripts/check-rust-verification-matrix.mjs --group gateway-service`
  - `node scripts/check-rust-verification-matrix.mjs --group admin-service`
  - `node scripts/check-rust-verification-matrix.mjs --group portal-service`
  - `node scripts/check-rust-verification-matrix.mjs --group product-runtime`
  - `node scripts/check-rust-verification-matrix.mjs --group workspace`
- the GitHub workflow now has a dedicated manual Windows workspace lane for future hosted proof collection

Still open:

- the new Windows `workspace` workflow lane has not yet produced its first hosted execution record in this workspace session
- cross-platform release/runtime confidence outside the local Windows workstation still requires Linux/macOS execution evidence
