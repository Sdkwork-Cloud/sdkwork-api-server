# Commerce Fulfillment Idempotency Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make portal commerce settlement replay-safe so the same order cannot re-apply quota, re-consume a live coupon, or re-activate membership side effects after retries or partial recovery.

**Architecture:** Add order-scoped settlement step guards in storage so each side effect is recorded and applied atomically in the same database transaction. Route paid settlement and any immediate fulfillment flow through the same order-aware finalization helper so retries resume safely instead of mutating state twice.

**Tech Stack:** Rust, sqlx, SQLite, Postgres, async trait object stores, cargo test

---

### Task 1: Reproduce the duplicate-settlement bug

**Files:**
- Create: `crates/sdkwork-api-app-commerce/tests/commerce_order_settlement.rs`
- Test: `crates/sdkwork-api-app-commerce/tests/commerce_order_settlement.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn restored_pending_order_replay_does_not_reapply_quota_or_reconsume_coupon() {
    // create a paid order with a live coupon
    // settle once
    // restore order status back to pending_payment
    // settle again
    // assert quota total and coupon remaining did not change twice
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --offline -p sdkwork-api-app-commerce restored_pending_order_replay_does_not_reapply_quota_or_reconsume_coupon -- --nocapture`
Expected: FAIL because settlement replays quota/coupon side effects or rejects the replay after the coupon was already consumed.

- [ ] **Step 3: Write minimal implementation**

```rust
// Add storage-backed settlement step guards:
// - quota step keyed by order_id
// - coupon step keyed by order_id
// - membership step keyed by order_id
// Each guard applies the effect and records completion in one transaction.
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --offline -p sdkwork-api-app-commerce restored_pending_order_replay_does_not_reapply_quota_or_reconsume_coupon -- --nocapture`
Expected: PASS

### Task 2: Route settlement through the guarded path

**Files:**
- Modify: `crates/sdkwork-api-app-commerce/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`

- [ ] **Step 1: Update the failing call path**

```rust
// Replace direct quota/coupon/membership mutations inside settle_portal_commerce_order
// with store-level guarded settlement step methods.
```

- [ ] **Step 2: Verify commerce tests stay green**

Run: `cargo test --offline -p sdkwork-api-app-commerce -- --nocapture`
Expected: PASS

### Task 3: Regress payment callback integration

**Files:**
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Test: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`

- [ ] **Step 1: Re-run callback coverage**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS

- [ ] **Step 2: Re-run HTTP callback coverage**

Run: `cargo test --offline -p sdkwork-api-interface-http --test payment_callbacks -- --nocapture`
Expected: PASS
