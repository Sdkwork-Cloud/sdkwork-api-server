# Payment Multi-Capture Aggregation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Support multiple distinct settlement capture transactions for one payment order while keeping fulfillment, refund ceilings, and audit state correct.

**Architecture:** Replace the single-sale assumption with an aggregation model: one sale row per accepted provider capture transaction, monotonic updates for same-transaction replays, aggregate `captured_amount_minor` from accepted sale rows, and trigger fulfillment only when the aggregate meets the payable threshold.

**Tech Stack:** Rust, sqlx, SQLite, Axum, cargo test

---

### Task 1: Lock the multi-capture regression with failing tests

**Files:**
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Write the failing app regression for accumulating distinct captures**

```rust
#[tokio::test]
async fn distinct_partial_captures_accumulate_without_conflict_until_threshold() {}
```

- [ ] **Step 2: Write the failing app regression for full fulfillment after aggregate threshold**

```rust
#[tokio::test]
async fn final_distinct_capture_crosses_threshold_and_fulfills_once() {}
```

- [ ] **Step 3: Write the failing HTTP regression for multi-capture accumulation**

```rust
#[tokio::test]
async fn payment_callback_route_accumulates_distinct_partial_captures_for_same_order() {}
```

- [ ] **Step 4: Write the failing portal regression for multiple sale rows**

```rust
#[tokio::test]
async fn portal_order_center_lists_multiple_capture_transactions_with_aggregated_refund_capacity() {}
```

- [ ] **Step 5: Run targeted tests to verify they fail**

Run: `cargo test --offline -p sdkwork-api-app-payment distinct_partial_captures_accumulate_without_conflict_until_threshold -- --nocapture`
Expected: FAIL because a second provider capture is still treated as a conflict instead of an accepted incremental capture.

Run: `cargo test --offline -p sdkwork-api-app-payment final_distinct_capture_crosses_threshold_and_fulfills_once -- --nocapture`
Expected: FAIL because aggregate captured money is not computed from multiple sale rows yet.

Run: `cargo test --offline -p sdkwork-api-interface-http payment_callback_route_accumulates_distinct_partial_captures_for_same_order --test payment_callbacks -- --nocapture`
Expected: FAIL because the route still preserves only one accepted sale row.

Run: `cargo test --offline -p sdkwork-api-interface-portal portal_order_center_lists_multiple_capture_transactions_with_aggregated_refund_capacity --test portal_order_center -- --nocapture`
Expected: FAIL because the order center cannot show multiple accepted sale rows for one order.

### Task 2: Implement multi-capture aggregation in payment callback processing

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`

- [ ] **Step 1: Replace single-sale settlement assumptions with per-provider-transaction sale identity**
- [ ] **Step 2: Accept a new sale row when a distinct provider transaction fits within the remaining payable amount**
- [ ] **Step 3: Keep same-provider-transaction replays monotonic**
- [ ] **Step 4: Recompute aggregate `captured_amount_minor` from accepted sale rows**
- [ ] **Step 5: Trigger fulfillment exactly once when the aggregate crosses the payable threshold**
- [ ] **Step 6: Keep refund ceilings aggregated across accepted sale rows**

- [ ] **Step 7: Re-run targeted app regressions**

Run: `cargo test --offline -p sdkwork-api-app-payment distinct_partial_captures_accumulate_without_conflict_until_threshold -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-app-payment final_distinct_capture_crosses_threshold_and_fulfills_once -- --nocapture`
Expected: PASS

### Task 3: Expose aggregated capture state through HTTP and portal projections

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Ensure HTTP callback flows preserve multiple accepted sale rows**
- [ ] **Step 2: Ensure portal order-center projections show multiple sale transactions and aggregate refund capacity**

- [ ] **Step 3: Re-run targeted interface regressions**

Run: `cargo test --offline -p sdkwork-api-interface-http payment_callback_route_accumulates_distinct_partial_captures_for_same_order --test payment_callbacks -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-interface-portal portal_order_center_lists_multiple_capture_transactions_with_aggregated_refund_capacity --test portal_order_center -- --nocapture`
Expected: PASS

### Task 4: Re-verify the affected payment slice

**Files:**
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Test: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`
- Test: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`
- Test: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`

- [ ] **Step 1: Run payment app tests**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS

- [ ] **Step 2: Run HTTP callback tests**

Run: `cargo test --offline -p sdkwork-api-interface-http --test payment_callbacks -- --nocapture`
Expected: PASS

- [ ] **Step 3: Run portal order-center tests**

Run: `cargo test --offline -p sdkwork-api-interface-portal --test portal_order_center -- --nocapture`
Expected: PASS

- [ ] **Step 4: Run admin payment tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS
