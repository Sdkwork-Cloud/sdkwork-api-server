# Payment Provider Conflict Audit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prevent duplicate sale transactions when a settled payment order is replayed with a conflicting provider transaction id, while persisting reconciliation evidence for operators.

**Architecture:** Extend verified payment callback settlement handling in `sdkwork-api-app-payment` so sale transaction creation reuses the existing canonical or legacy sale transaction for a payment order. When a replay carries a different provider transaction id, keep the original local sale transaction untouched and emit deterministic reconciliation evidence instead of inserting a new sale row.

**Tech Stack:** Rust, sqlx, SQLite, Postgres, cargo test

---

### Task 1: Reproduce the payment-side replay gap

**Files:**
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`

- [ ] **Step 1: Write the failing regression test**

```rust
#[tokio::test]
async fn settled_payment_replay_keeps_single_sale_transaction_and_records_provider_conflict() {}
```

- [ ] **Step 2: Run the targeted test to verify it fails**

Run: `cargo test --offline -p sdkwork-api-app-payment settled_payment_replay_keeps_single_sale_transaction_and_records_provider_conflict -- --nocapture`
Expected: FAIL because the replay creates a second sale transaction or does not persist reconciliation evidence for the conflicting provider transaction id.

### Task 2: Reuse canonical sale transactions and persist conflict evidence

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`

- [ ] **Step 1: Add a sale transaction reuse helper that recognizes canonical and legacy sale rows**
- [ ] **Step 2: Create deterministic payment conflict batch and line ids**
- [ ] **Step 3: Persist reconciliation evidence when a replayed settlement carries a different provider transaction id**
- [ ] **Step 4: Update callback hydration to fall back to the canonical sale transaction when exact provider-id lookup misses**

- [ ] **Step 5: Re-run the targeted regression test**

Run: `cargo test --offline -p sdkwork-api-app-payment settled_payment_replay_keeps_single_sale_transaction_and_records_provider_conflict -- --nocapture`
Expected: PASS

### Task 3: Re-verify payment and callback surfaces

**Files:**
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Test: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`
- Test: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`

- [ ] **Step 1: Run payment app tests**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS

- [ ] **Step 2: Run HTTP payment callback tests**

Run: `cargo test --offline -p sdkwork-api-interface-http --test payment_callbacks -- --nocapture`
Expected: PASS

- [ ] **Step 3: Run admin payment tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test admin_payments -- --nocapture`
Expected: PASS
