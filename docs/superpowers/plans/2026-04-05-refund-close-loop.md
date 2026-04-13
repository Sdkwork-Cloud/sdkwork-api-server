# Refund Close-Loop Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a canonical refund loop for paid recharge orders with refund order records, quota reversal, account-history reversal, and finance journal evidence.

**Architecture:** Build refund orchestration in `sdkwork-api-app-payment`, reuse deterministic ids for refund transactions and journals, and add transactional refund-step guards in storage for the two mutable side effects: quota reversal and account grant reversal.

**Tech Stack:** Rust, sqlx, SQLite, Postgres, cargo test

---

### Task 1: Reproduce the missing refund close loop

**Files:**
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`

- [ ] **Step 1: Write the failing refund orchestration tests**

```rust
#[tokio::test]
async fn recharge_refund_success_reverses_quota_and_account_history_once() {}

#[tokio::test]
async fn partial_recharge_refund_marks_payment_order_partially_refunded() {}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --offline -p sdkwork-api-app-payment recharge_refund -- --nocapture`
Expected: FAIL because no refund orchestration exists yet.

### Task 2: Add refund application services

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`

- [ ] **Step 1: Implement refund request creation**

```rust
pub async fn request_payment_order_refund(...) -> Result<RefundOrderRecord>
```

- [ ] **Step 2: Implement successful refund finalization**

```rust
pub async fn finalize_refund_order_success(...) -> Result<RefundCloseLoopResult>
```

- [ ] **Step 3: Implement account grant reversal helper**

```rust
pub async fn reverse_commerce_order_account_grant(...) -> Result<...>
```

- [ ] **Step 4: Re-run the new refund tests**

Run: `cargo test --offline -p sdkwork-api-app-payment recharge_refund -- --nocapture`
Expected: PASS

### Task 3: Add transactional refund-step storage guards

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`

- [ ] **Step 1: Add refund step methods to the storage traits**
- [ ] **Step 2: Add SQLite implementations**
- [ ] **Step 3: Add Postgres implementations**
- [ ] **Step 4: Re-run trait coverage**

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test admin_store_trait -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-storage-postgres --test admin_store_trait -- --nocapture`
Expected: PASS

### Task 4: Re-verify payment and HTTP regression surface

**Files:**
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Test: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`

- [ ] **Step 1: Run payment tests**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS

- [ ] **Step 2: Run HTTP callback tests**

Run: `cargo test --offline -p sdkwork-api-interface-http --test payment_callbacks -- --nocapture`
Expected: PASS
