# 2026-04-09 Workspace Test Module Visibility Review

## Scope

- Loop target: restore a green Rust workspace verification gate under the repeat-step execution contract.
- Architecture reference: `docs/架构/132-Provider接入与适配器治理设计-2026-04-07.md`.
- This iteration fixed test-module boundary defects only. No provider runtime, protocol routing, billing, or control-plane behavior was changed.

## Findings

### P0 - split Rust test targets reused private sibling fixtures

- Affected targets:
  - `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision`
  - `crates/sdkwork-api-app-gateway/tests/extension_dispatch`
- Pattern:
  - parent `mod.rs` imported `use support::*;`
  - child modules referenced helpers, guards, fixtures, and shared state from `support.rs`
  - those items remained private, so Rust privacy blocked sibling access
- Symptom:
  - first-stage failures were `E0425`, `E0433`, and `E0616`
  - later `E0282` inference errors were secondary fallout after shared helpers disappeared from scope
- Business impact:
  - `cargo test --workspace --no-run -j 1` failed
  - startup warm-up and release-quality verification were blocked
  - regression confidence for runtime reload and extension dispatch paths was reduced

## Root Cause

- Test files had been decomposed into child modules, but the old single-file visibility assumption was kept.
- In Rust, `use support::*;` at the parent level does not bypass privacy for sibling modules.
- Shared helpers needed module-scoped exports such as `pub(super)`.

## Fix

### `sdkwork-api-app-runtime`

- File: `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision/support.rs`
- Action:
  - promoted shared helper functions, wait helpers, config writers, and guard types to `pub(super)`
  - kept the change limited to test support boundaries

### `sdkwork-api-app-gateway`

- File: `crates/sdkwork-api-app-gateway/tests/extension_dispatch/support.rs`
- Action:
  - promoted shared fixture helpers, extension-env guards, native-dynamic helpers, and log guards to `pub(super)`
  - promoted `UpstreamCaptureState.authorization` to `pub(super)` because sibling tests assert captured auth behavior directly

## Architecture Alignment

- The fix does not alter provider integration rules.
- OpenAI, Anthropic, and Gemini compatible protocols remain passthrough-oriented.
- Plugin conversion remains reserved for heterogeneous provider APIs, consistent with `docs/架构/132-Provider接入与适配器治理设计-2026-04-07.md`.
- This iteration improves verification infrastructure so the plugin/passthrough architecture can be validated reliably.

## Verification

- `cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision --no-run -j 1`
- `cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision -j 1`
- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch --no-run -j 1`
- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch -j 1`
- `cargo test --workspace --no-run -j 1`

## Result

- `standalone_runtime_supervision`: 16 passed
- `extension_dispatch`: 14 passed
- workspace `--no-run` gate: passed

## Residual Gaps

- `crates/sdkwork-api-storage-postgres/tests/integration_postgres/routing_usage.rs` still emits one dead-code warning for `postgres_store_persists_quota_policies_when_url_is_provided`.
- Full workspace runtime execution was not run in this iteration; the workspace gate validated compile/link readiness, not every test binary end-to-end.
- `bin/start-dev.ps1` was not re-run in this iteration after the test-gate repair.

## Next Step

1. Remove or justify the remaining dead-code warning so the verification surface is cleaner.
2. Run the next practical smoke layer after the compile gate, prioritizing Windows startup/runtime verification.
3. Continue the step loop from the first new blocker instead of widening scope without evidence.
