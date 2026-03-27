# Router Portal Dual-Mode Host Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a shared product host so `sdkwork-router-portal` can run as both a Tauri desktop app and a server-side CLI/service while hosting web UI plus router APIs.

**Architecture:** Add a new shared Rust product-runtime crate above the existing API services and runtime host. Use it from both Tauri shells and a new server service crate. Keep cluster support by exposing role-based topology and reusing DB-backed rollout supervision.

**Tech Stack:** Rust, Tauri 2, Axum, Pingora, existing `sdkwork-api-*` crates, Vite/React

---

### Task 1: Make low-level runtime pieces support dynamic product hosting

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Modify: `crates/sdkwork-api-config/src/lib.rs`
- Modify: `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision.rs`
- Modify: `crates/sdkwork-api-config/tests/config_loading.rs`

- [ ] **Step 1: Write failing tests for ephemeral listener binds and loader override cloning**
- [ ] **Step 2: Run targeted tests and verify they fail for the expected missing behavior**
- [ ] **Step 3: Implement actual-bind capture for `StandaloneListenerHost` and override cloning on `StandaloneConfigLoader`**
- [ ] **Step 4: Run targeted tests and verify they pass**

### Task 2: Add shared product runtime orchestration crate

**Files:**
- Create: `crates/sdkwork-api-product-runtime/Cargo.toml`
- Create: `crates/sdkwork-api-product-runtime/src/lib.rs`
- Create: `crates/sdkwork-api-product-runtime/tests/product_runtime.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Write failing product-runtime integration and unit tests**
- [ ] **Step 2: Run targeted product-runtime tests and verify they fail**
- [ ] **Step 3: Implement role planning, local service bootstrap, runtime supervision wiring, and public web host startup**
- [ ] **Step 4: Run targeted product-runtime tests and verify they pass**

### Task 3: Add server-mode product service

**Files:**
- Create: `services/router-product-service/Cargo.toml`
- Create: `services/router-product-service/src/main.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Write a minimal smoke-oriented server bootstrap around the shared product runtime**
- [ ] **Step 2: Verify the service compiles**
- [ ] **Step 3: Ensure env-driven role slicing and upstream override behavior is wired in**

### Task 4: Move Tauri desktop shells onto the shared product runtime

**Files:**
- Modify: `apps/sdkwork-router-portal/src-tauri/src/main.rs`
- Modify: `apps/sdkwork-router-portal/src-tauri/Cargo.toml`
- Modify: `apps/sdkwork-router-portal/src-tauri/tauri.conf.json`
- Modify: `apps/sdkwork-router-admin/src-tauri/src/main.rs`
- Modify: `apps/sdkwork-router-admin/src-tauri/Cargo.toml`
- Modify: `apps/sdkwork-router-admin/src-tauri/tauri.conf.json`

- [ ] **Step 1: Write the runtime bootstrap adapter that resolves bundled resource dirs or workspace fallbacks**
- [ ] **Step 2: Replace hard-coded `RuntimeHostConfig::new(...)` startup with shared product runtime startup**
- [ ] **Step 3: Bundle both portal/admin dist assets into each Tauri package**
- [ ] **Step 4: Fix portal dev URL and keep existing IPC commands intact**

### Task 5: Expose server-mode workflow from the portal app surface

**Files:**
- Modify: `apps/sdkwork-router-portal/package.json`
- Modify: `apps/sdkwork-router-portal/README.md`
- Modify: `apps/sdkwork-router-admin/README.md`
- Create: `scripts/build-router-desktop-assets.mjs`

- [ ] **Step 1: Add scripts for server start and dual-frontend desktop asset builds**
- [ ] **Step 2: Document desktop mode, server mode, and cluster/topology envs**
- [ ] **Step 3: Verify package-level scripts are coherent with the new runtime**

### Task 6: Verify and polish

**Files:**
- Modify: `docs/architecture/software-architecture.md`

- [ ] **Step 1: Run targeted Rust tests for config, app-runtime, and product-runtime**
- [ ] **Step 2: Run server service compile checks**
- [ ] **Step 3: Run portal/admin typecheck**
- [ ] **Step 4: Update architecture docs to reflect the new product host model**
