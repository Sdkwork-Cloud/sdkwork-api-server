# Provider Protocol Step 3 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** unify stateful protocol preview and translated execution under one shared planned execution object so mixed-protocol routing cannot re-select a different provider.

**Architecture:** keep routing ownership inside `sdkwork-api-app-gateway`. Add execution entrypoints that consume a precomputed `PlannedExecutionProviderContext`, persist the initial decision log once, and execute against that exact provider plan. Update HTTP compat handlers to reuse the same plan for passthrough and translated fallback.

**Tech Stack:** Rust, Axum test upstreams, SQLite admin store, app-gateway routing/execution helpers

---

### Task 1: Red Test For Planned Execution Reuse

**Files:**
- Create: `crates/sdkwork-api-app-gateway/tests/planned_execution.rs`

- [x] Step 1: write a failing gateway test that creates a no-log planned provider context, executes a chat completion from that plan, and asserts exactly one routing decision log is persisted.
- [x] Step 2: run `cargo test -p sdkwork-api-app-gateway --test planned_execution` and verify it fails because the planned execution helper does not exist yet.

### Task 2: Planned Execution Entry Points

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/gateway_routing.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/gateway_types.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/relay_chat.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/relay_responses.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`

- [x] Step 1: add an optional seeded no-log planner variant for deterministic tests.
- [x] Step 2: add chat completion and count-tokens execution helpers that consume `PlannedExecutionProviderContext`, persist the selected-provider decision log once, and execute from the same plan.
- [x] Step 3: keep failover behavior aligned with existing execution helpers.
- [x] Step 4: rerun `cargo test -p sdkwork-api-app-gateway --test planned_execution` and make it pass.

### Task 3: Wire HTTP Compat To Shared Plan

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/gateway_compat_handlers.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

- [x] Step 1: update stateful Anthropic/Gemini compat handlers to obtain one planned provider context.
- [x] Step 2: use raw passthrough when protocol matches, otherwise execute translated fallback from the same planned context.
- [x] Step 3: avoid re-selecting a second provider in these fallback branches.

### Task 4: Verify And Document

**Files:**
- Modify: `docs/架构/165-provider-protocol-kind-plugin-runtime-split-2026-04-09.md`
- Modify: `docs/review/2026-04-09-provider-protocol-plugin-runtime-split-review.md`
- Modify: `docs/step/2026-04-09-provider-protocol-plugin-runtime-split-step-update.md`
- Modify: `docs/release/2026-04-09-unreleased-provider-protocol-plugin-runtime-split.md`
- Modify: `docs/release/CHANGELOG.md`

- [x] Step 1: run `cargo test -p sdkwork-api-app-gateway --test planned_execution`.
- [x] Step 2: run `cargo test -p sdkwork-api-interface-http --test anthropic_messages_route`.
- [x] Step 3: run `cargo test -p sdkwork-api-interface-http --test gemini_generate_content_route`.
- [x] Step 4: run `cargo check -p sdkwork-api-app-gateway -p sdkwork-api-interface-http`.
- [x] Step 5: update architecture/review/release/step docs with the unified planned execution result and remaining gaps.
