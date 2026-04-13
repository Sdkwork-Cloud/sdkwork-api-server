# Refund Finalization Recovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make refund success replay converge to one canonical refund transaction even if `provider_refund_id` changes while the refund order is still non-terminal.

**Architecture:** Keep balance reversals guarded by refund processing steps, keep finance evidence anchored by deterministic journal ids, and move refund payment transaction identity to a refund-order-based id with replay-aware reuse.

**Tech Stack:** Rust, sqlx, SQLite, Postgres, cargo test

---

### Task 1: Reproduce the replay-duplication gap

**Files:**
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`

- [ ] **Step 1: Write the failing replay regression test**

```rust
#[tokio::test]
async fn refund_processing_replay_keeps_single_refund_transaction() {}
```

- [ ] **Step 2: Run the targeted test to verify it fails**

Run: `cargo test --offline -p sdkwork-api-app-payment refund_processing_replay_keeps_single_refund_transaction -- --nocapture`
Expected: FAIL because the replay creates a second refund transaction row.

### Task 2: Canonicalize refund transaction identity

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`

- [ ] **Step 1: Introduce a refund-order-based transaction id helper**
- [ ] **Step 2: Update refund transaction construction to use the refund order id**
- [ ] **Step 3: Reuse an existing refund transaction row during replay instead of overwriting it**

- [ ] **Step 4: Re-run the targeted regression test**

Run: `cargo test --offline -p sdkwork-api-app-payment refund_processing_replay_keeps_single_refund_transaction -- --nocapture`
Expected: PASS

### Task 3: Re-verify refund and portal payment surfaces

**Files:**
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_order_service.rs`
- Test: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Run payment app tests**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS

- [ ] **Step 2: Run portal regression tests**

Run: `cargo test --offline -p sdkwork-api-interface-portal -- --nocapture`
Expected: PASS
