# Extension Discovery Runtime Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Turn the current extension manifest and instance model into a real configuration-driven runtime by adding filesystem discovery, runtime-policy filtering, and gateway host wiring for externally declared provider extensions.

**Architecture:** Keep built-in provider extensions as the fast path, but add a second manifest source: discovered extension packages from configured directories. The extension host will parse standardized package manifests, filter them through an explicit runtime policy, and expose them to the gateway. The gateway will then map discovered provider manifests onto existing protocol adapters when the manifest declares a supported protocol, so connector-style and future native-dynamic packages can participate in runtime planning and relay execution without breaking the current layering.

**Tech Stack:** Rust, Axum, serde, toml, sqlx, std::path, existing extension host and gateway crates

---

### Task 1: Add failing tests for extension package discovery and runtime policy

**Files:**
- Create: `crates/sdkwork-api-extension-host/tests/discovery.rs`
- Modify: `crates/sdkwork-api-config/tests/config_loading.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- the extension host can discover `sdkwork-extension.toml` manifests from configured directories
- the discovery layer filters manifests when a runtime is disabled by policy
- `StandaloneConfig` parses extension search paths and runtime toggles from environment

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-extension-host --test discovery -q`
- `cargo test -p sdkwork-api-config --test config_loading -q`

Expected: FAIL because the host has no filesystem discovery API and runtime config has no extension discovery fields.

### Task 2: Implement standardized extension manifest discovery

**Files:**
- Modify: `crates/sdkwork-api-extension-core/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`

**Step 1: Add additive manifest metadata**

Extend `ExtensionManifest` with additive protocol metadata so discovered provider extensions can bind to an existing protocol adapter without inventing a second runtime model.

**Step 2: Add discovery policy and manifest parsing**

Implement host-side discovery that:

- scans configured directories recursively for `sdkwork-extension.toml`
- deserializes manifests from TOML
- rejects unsupported runtimes according to policy
- returns deterministic ordering for test stability

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-extension-host --test discovery -q`
- `cargo test -p sdkwork-api-extension-host -q`

Expected: PASS

### Task 3: Wire discovered provider manifests into gateway execution

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`

**Step 1: Write the failing end-to-end test**

Add a regression proving that:

- a provider whose `extension_id` is not built in can still relay if its manifest is discovered from a configured directory
- the discovered manifest can declare a supported protocol such as `openai`
- persisted installation and instance state still override base URL and enablement

**Step 2: Implement gateway host wiring**

Add a helper that builds an extension host from:

- built-in manifests
- discovered manifests from configured filesystem paths
- persisted installations and instances from storage

Map supported discovered provider protocols onto the current adapter constructors.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch -q`
- `cargo test -p sdkwork-api-interface-http --test chat_route -q`

Expected: PASS

### Task 4: Add runtime config and update architecture docs

**Files:**
- Modify: `crates/sdkwork-api-config/src/lib.rs`
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`

**Step 1: Add additive runtime config**

Expose:

- extension search paths
- connector runtime enablement
- native dynamic runtime enablement

Do not break current defaults.

**Step 2: Update docs to reflect real capability**

Document what is now implemented:

- manifest discovery
- configuration-driven loading
- discovered provider relay through supported protocols

Also document what still remains planned:

- native dynamic ABI execution lifecycle
- out-of-process connector health supervision

**Step 3: Run full verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace -q`

Expected: PASS

**Step 4: Commit**

```bash
git add docs/plans/2026-03-14-extension-discovery-runtime-implementation.md crates/sdkwork-api-extension-core crates/sdkwork-api-extension-host crates/sdkwork-api-app-gateway crates/sdkwork-api-config README.md docs/architecture/runtime-modes.md
git commit -m "feat: add extension discovery runtime wiring"
```
