# Payment Reconciliation Metrics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose payment reconciliation anomaly totals and active reason gauges from the admin Prometheus metrics endpoint.

**Architecture:** Reuse admin reconciliation summary aggregation, append a Prometheus-formatted reconciliation section to the stateful admin `/metrics` response, and verify the emitted gauges through integration tests.

**Tech Stack:** Rust, Axum, Prometheus text format, SQLite-backed admin integration tests, cargo test

---

### Task 1: Lock reconciliation metric exposure with failing tests

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`

- [ ] **Step 1: Add a failing `/metrics` reconciliation gauge regression**
- [ ] **Step 2: Assert lifecycle totals and active-reason gauge output**

- [ ] **Step 3: Run targeted admin payment tests to verify failure**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: FAIL because `/metrics` does not yet expose payment reconciliation gauges.

### Task 2: Implement Prometheus rendering for reconciliation metrics

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`

- [ ] **Step 1: Add a Prometheus renderer for reconciliation summary data**
- [ ] **Step 2: Extend the stateful admin `/metrics` route to append reconciliation metrics**
- [ ] **Step 3: Reuse summary aggregation so JSON and metrics stay consistent**

- [ ] **Step 4: Re-run targeted admin payment tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS

### Task 3: Re-verify the affected payment/admin slice

**Files:**
- Test: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`

- [ ] **Step 1: Re-run admin payment tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS

- [ ] **Step 2: Re-run payment application regressions**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS
