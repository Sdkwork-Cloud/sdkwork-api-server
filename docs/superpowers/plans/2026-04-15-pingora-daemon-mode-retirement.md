# Pingora Daemon Mode Retirement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** retire vendored Pingora daemon mode so the product runtime uses a single supported production model: foreground execution managed by scripts or an external service manager.

**Architecture:** treat `daemon=true` as an unsupported legacy flag at configuration validation time instead of a runtime fork path. Remove the vendored `daemonize` crate and its helper module, keep configuration fields long enough to emit actionable migration errors, and align docs plus dependency-audit regression tests with the new supported runtime shape.

**Tech Stack:** Rust vendored `pingora-core`, Cargo.lock dependency hygiene, Node test harnesses, repository docs for managed runtime scripts.

**Closure Update:** this plan is now complete. `daemonize` is retired from the active workspace graph, and the current workspace `cargo audit --json --no-fetch --stale` result is clean. Any references below to the old `paste` and `rand` warning paths are historical execution context, not active residual advisories.

---

## File Map

- Modify: `vendor/pingora-core-0.8.0/src/server/configuration/mod.rs`
  - reject `daemon=true` during config validation and add regression tests for YAML and CLI override paths.
- Modify: `vendor/pingora-core-0.8.0/src/server/mod.rs`
  - remove runtime daemonization path and update comments to reflect the supported foreground-only model.
- Delete: `vendor/pingora-core-0.8.0/src/server/daemon.rs`
  - remove the retired daemon helper implementation.
- Modify: `vendor/pingora-core-0.8.0/Cargo.toml`
  - drop the `daemonize` dependency.
- Modify: `scripts/check-rust-dependency-audit.test.mjs`
  - prevent `daemonize` from re-entering the lockfile.
- Modify: `docs/getting-started/runtime-modes.md`
  - replace “direct daemon use” wording with foreground plus service-manager guidance.
- Modify: `docs/zh/getting-started/runtime-modes.md`
  - keep the Chinese runtime-mode guidance aligned with the English source.
- Modify: `docs/getting-started/script-lifecycle.md`
  - remove “service or daemon” wording and standardize on service-manager foreground guidance.
- Modify: `Cargo.lock`
  - record the dependency graph after daemon retirement.

### Task 1: Write the failing regression tests

**Files:**
- Modify: `vendor/pingora-core-0.8.0/src/server/configuration/mod.rs`
- Modify: `scripts/check-rust-dependency-audit.test.mjs`

- [ ] **Step 1: Add a config validation test for YAML daemon mode**

Write a Rust test that loads:

```yaml
---
version: 1
daemon: true
```

and asserts `ServerConf::from_yaml(...)` returns an error whose message tells operators to run in foreground with a service manager.

- [ ] **Step 2: Add a config validation test for CLI override daemon mode**

Write a Rust test that constructs `Opt { daemon: true, ..Default::default() }`, calls `ServerConf::new_with_opt_override(&opt)`, and asserts the result is `None` or otherwise rejected according to the chosen validation path.

- [ ] **Step 3: Add a dependency-audit lockfile regression**

Extend `scripts/check-rust-dependency-audit.test.mjs` so the lockfile assertion also rejects:

```text
name = "daemonize"
version = "0.5.0"
```

- [ ] **Step 4: Run the narrow failing tests**

Run:

```bash
cargo test --manifest-path vendor/pingora-core-0.8.0/Cargo.toml test_reject_daemon_mode -- --nocapture
node --test scripts/check-rust-dependency-audit.test.mjs
```

Expected: both fail before production code changes because daemon mode is still accepted and `daemonize` is still pinned in `Cargo.lock`.

### Task 2: Retire daemon mode in vendored Pingora

**Files:**
- Modify: `vendor/pingora-core-0.8.0/src/server/configuration/mod.rs`
- Modify: `vendor/pingora-core-0.8.0/src/server/mod.rs`
- Delete: `vendor/pingora-core-0.8.0/src/server/daemon.rs`
- Modify: `vendor/pingora-core-0.8.0/Cargo.toml`

- [ ] **Step 1: Reject daemon mode during configuration validation**

Implement the smallest validation that refuses `daemon=true` with an actionable migration message:

- foreground execution is the only supported mode
- use `bin/start.* --foreground`
- or use systemd / launchd / Windows Task Scheduler / Kubernetes

- [ ] **Step 2: Ensure CLI override paths also validate**

Update `load_yaml_with_opt_override`, `new_with_opt_override`, and any merged-conf constructor so `opt.daemon = true` cannot bypass validation.

- [ ] **Step 3: Remove runtime daemonization**

Delete the Unix daemon module import and call sites from `vendor/pingora-core-0.8.0/src/server/mod.rs`. Update comments so they no longer describe fork-based daemon behavior as supported.

- [ ] **Step 4: Remove the dependency and regenerate the lockfile**

Drop `daemonize` from `vendor/pingora-core-0.8.0/Cargo.toml` and let Cargo rewrite `Cargo.lock` so the dependency disappears.

- [ ] **Step 5: Run the narrow tests again**

Run:

```bash
cargo test --manifest-path vendor/pingora-core-0.8.0/Cargo.toml test_reject_daemon_mode -- --nocapture
node --test scripts/check-rust-dependency-audit.test.mjs
```

Expected: both pass.

### Task 3: Align operator-facing docs

**Files:**
- Modify: `docs/getting-started/runtime-modes.md`
- Modify: `docs/zh/getting-started/runtime-modes.md`
- Modify: `docs/getting-started/script-lifecycle.md`

- [ ] **Step 1: Replace stale daemon wording**

Update runtime and lifecycle docs so they consistently state:

- application processes run in foreground
- service managers own backgrounding and restart policy
- managed scripts support foreground mode for supervised deployments

- [ ] **Step 2: Keep English and Chinese docs consistent**

Do not leave one locale promising daemon mode while the other deprecates it.

### Task 4: Final verification

**Files:**
- Modify: none unless verification exposes another defect

- [ ] **Step 1: Run focused verification**

Run:

```bash
node --test scripts/check-rust-dependency-audit.test.mjs scripts/check-rust-verification-matrix.test.mjs scripts/rust-verification-workflow.test.mjs
node scripts/check-rust-verification-matrix.mjs --group dependency-audit
cargo audit --json --no-fetch --stale
cargo check -p sdkwork-api-runtime-host
```

Expected:

- dependency-audit tests pass
- `dependency-audit` matrix group passes
- `cargo audit` reports zero vulnerabilities
- `daemonize` no longer appears in remaining informational warnings

- [ ] **Step 2: Record closure status honestly**

After daemon retirement, document that the daemon-specific dependency path is closed and that later hardening slices also removed the formerly tracked `paste` and `rand` warning paths from the active workspace graph.
