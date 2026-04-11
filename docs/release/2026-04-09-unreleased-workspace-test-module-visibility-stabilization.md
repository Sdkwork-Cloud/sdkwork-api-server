# Unreleased - Workspace Test Module Visibility Stabilization

- Date: 2026-04-09
- Type: patch
- Scope: Rust workspace verification, runtime supervision tests, extension dispatch tests
- Highlights:
  - fixed sibling-module visibility defects in `sdkwork-api-app-runtime/tests/standalone_runtime_supervision/support.rs`
  - fixed sibling-module visibility defects in `sdkwork-api-app-gateway/tests/extension_dispatch/support.rs`
  - restored a green `cargo test --workspace --no-run -j 1` gate
  - verified `standalone_runtime_supervision` at `16 / 16` and `extension_dispatch` at `14 / 14`
  - preserved the provider architecture contract: no passthrough or plugin runtime behavior changed
- Verification:
  - `cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision -j 1`
  - `cargo test -p sdkwork-api-app-gateway --test extension_dispatch -j 1`
  - `cargo test --workspace --no-run -j 1`
- Known gaps:
  - one dead-code warning remains in `sdkwork-api-storage-postgres` integration tests
  - startup smoke was not re-run in this iteration
