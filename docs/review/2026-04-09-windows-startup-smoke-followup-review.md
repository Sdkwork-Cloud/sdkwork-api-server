# 2026-04-09 Windows Startup Smoke Follow-up Review

## Scope

- Loop target: raise verification from Rust workspace compile readiness to Windows startup/runtime tooling readiness.
- Related architecture:
  - `docs/架构/132-Provider接入与适配器治理设计-2026-04-07.md`
- This iteration did not change provider passthrough/plugin behavior. It only tightened verification and removed a remaining test-entry warning.

## Findings

### P1 - one Postgres integration test had silently dropped out of execution

- File: `crates/sdkwork-api-storage-postgres/tests/integration_postgres/routing_usage.rs`
- Symptom:
  - workspace verification emitted one dead-code warning for `postgres_store_persists_quota_policies_when_url_is_provided`
- Root cause:
  - the async function was intended as an integration test but was missing `#[tokio::test]`
- Fix:
  - restored the missing `#[tokio::test]` attribute so the case is compiled as a real test entrypoint instead of unused code

### P1 - startup tooling contracts are green, the underlying managed workspace launch is green, but the direct PowerShell wrapper is still policy-blocked in this session

- Verified facts:
  - `start-dev.ps1` script structure and managed cargo warm-up contract are green through Node-based smoke coverage
  - `start-workspace` planning and process-supervision contracts are green
  - runtime-tooling contracts for start/stop wrappers, managed state, bind preflight, and preview/browser mode defaults are green
  - a real managed preview launch/stop cycle through `node scripts/dev/start-workspace.mjs` completed successfully with fresh temporary ports and runtime state
- Remaining blocker:
  - direct `PowerShell -> start-dev.ps1 -DryRun` execution from the current Codex shell environment was rejected by session policy before script execution
- Impact:
  - repository-owned startup logic is verified
  - this session now has a fresh managed launch/stop proof for the underlying runtime path
  - this session still lacks a fresh direct PowerShell wrapper proof for `bin/start-dev.ps1`

## Code Changes

- `crates/sdkwork-api-storage-postgres/tests/integration_postgres/routing_usage.rs`
  - added `#[tokio::test]` to `postgres_store_persists_quota_policies_when_url_is_provided`

## Verification

- `cargo test -p sdkwork-api-storage-postgres --test integration_postgres --no-run -j 1`
- `cargo test --workspace --no-run -j 1`
- `node --test --experimental-test-isolation=none bin/tests/start-dev-windows-backend-warmup.test.mjs`
- `node --test --experimental-test-isolation=none scripts/dev/tests/start-workspace.test.mjs scripts/dev/tests/process-supervision.test.mjs scripts/dev/tests/windows-rust-toolchain-guard.test.mjs`
- `node --test --experimental-test-isolation=none bin/tests/router-runtime-tooling.test.mjs`
- real managed launch smoke via inline Node harness executing:
  - `node scripts/dev/start-workspace.mjs --preview --database-url <temp-sqlite> --gateway-bind <temp> --admin-bind <temp> --portal-bind <temp> --web-bind <temp> --stop-file <temp>`

## Result

- workspace Rust no-run gate: passed
- startup-related Node smoke:
  - `start-dev-windows-backend-warmup`: `1 pass / 1 skip / 0 fail`
  - `start-workspace + process supervision + windows rust guard`: `17 / 17`
  - `router-runtime-tooling`: `38 pass / 8 skip / 0 fail`
- managed preview workspace smoke: passed with fresh temp runtime state and fresh loopback ports

## Residual Gaps

- direct `start-dev.ps1` execution remains unverified in this session because the shell blocks PowerShell child-execution policy transitions before the script body runs
- the underlying managed workspace launch path is verified, but the PowerShell wrapper layer still needs a host shell that allows direct script execution

## Next Step

1. Use a host shell that allows direct PowerShell script execution to run `bin/start-dev.ps1 -DryRun` and confirm the wrapper layer on top of the already-green managed runtime path.
2. Keep Node contract tests and the managed `start-workspace` smoke as the repository-owned fallback verification lanes when direct PowerShell execution is environment-blocked.
3. Continue the repeat-step loop from the next highest-value cross-platform runtime or release-governance verification gap.
