# Refund Provider Conflict Audit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Record reconciliation evidence when the same refund order is replayed with a conflicting provider refund id while preserving idempotent refund settlement.

**Architecture:** Extend the refund transaction reuse path in `sdkwork-api-app-payment` so existing refund transactions remain canonical, and conflicting provider refund ids are written as deterministic reconciliation records keyed by refund order and conflicting provider id.

**Tech Stack:** Rust, sqlx, SQLite, Postgres, cargo test

---

### Task 1: Reproduce the missing conflict audit

**Files:**
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`

- [ ] **Step 1: Write the failing regression test**

```rust
#[tokio::test]
async fn refund_processing_replay_keeps_single_refund_transaction() {}
```

- [ ] **Step 2: Run the targeted test to verify it fails**

Run: `cargo test --offline -p sdkwork-api-app-payment refund_processing_replay_keeps_single_refund_transaction -- --nocapture`
Expected: FAIL because no reconciliation evidence is persisted for the conflicting provider refund id.

### Task 2: Persist refund conflict reconciliation evidence

**Files:**
- Modify: `crates/sdkwork-api-domain-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`

- [ ] **Step 1: Add a reconciliation match status for provider-reference conflicts**
- [ ] **Step 2: Build deterministic refund conflict batch and line ids**
- [ ] **Step 3: Persist reconciliation evidence when a reused refund transaction sees a different provider refund id**

- [ ] **Step 4: Re-run the targeted regression test**

Run: `cargo test --offline -p sdkwork-api-app-payment refund_processing_replay_keeps_single_refund_transaction -- --nocapture`
Expected: PASS

### Task 3: Re-verify payment and operator surfaces

**Files:**
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Test: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`
- Test: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`

- [ ] **Step 1: Run payment app tests**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS

- [ ] **Step 2: Run admin payment tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS

- [ ] **Step 3: Run HTTP payment callback tests**

Run: `cargo test --offline -p sdkwork-api-interface-http --test payment_callbacks -- --nocapture`
Expected: PASS
