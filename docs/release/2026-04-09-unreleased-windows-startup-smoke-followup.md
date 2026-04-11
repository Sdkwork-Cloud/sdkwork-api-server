# Unreleased - Windows Startup Smoke Follow-up

- Date: 2026-04-09
- Type: patch
- Scope: startup verification, Windows runtime tooling, Postgres integration tests
- Highlights:
  - restored `postgres_store_persists_quota_policies_when_url_is_provided` as a real Postgres integration test by adding the missing `#[tokio::test]`
  - re-verified the Rust workspace `--no-run` gate after the test-entry repair
  - re-verified the repository-owned Windows startup contracts through `start-dev` warm-up, `start-workspace`, process-supervision, rust-toolchain-guard, and runtime-tooling Node smoke suites
  - executed a fresh managed preview launch/stop smoke through `node scripts/dev/start-workspace.mjs` with temporary runtime state, temporary ports, and green gateway/admin/portal/web readiness checks
  - kept architecture truth intact: no provider passthrough or plugin-adapter runtime behavior changed
- Verification:
  - `cargo test -p sdkwork-api-storage-postgres --test integration_postgres --no-run -j 1`
  - `cargo test --workspace --no-run -j 1`
  - `node --test --experimental-test-isolation=none bin/tests/start-dev-windows-backend-warmup.test.mjs`
  - `node --test --experimental-test-isolation=none scripts/dev/tests/start-workspace.test.mjs scripts/dev/tests/process-supervision.test.mjs scripts/dev/tests/windows-rust-toolchain-guard.test.mjs`
  - `node --test --experimental-test-isolation=none bin/tests/router-runtime-tooling.test.mjs`
- Known gaps:
  - direct `PowerShell -> bin/start-dev.ps1` execution is still blocked by this shell session's policy boundary
  - the underlying managed runtime path is proven, but the PowerShell wrapper layer itself still lacks a fresh direct execution proof in this session
