# Payment Authorization Lifecycle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a first-class `authorized` payment lifecycle so verified authorization callbacks advance payment state safely without triggering fulfillment until capture settles.

**Architecture:** Extend the payment domain enums and callback normalization path to recognize authorization as an intermediate payment outcome. Persist a canonical authorization transaction for auditability, keep fulfillment and account side effects gated on settlement only, and expose the new state through order-center and gateway callback surfaces.

**Tech Stack:** Rust, sqlx, SQLite, Axum, cargo test

---

### Task 1: Reproduce the missing authorization lifecycle

**Files:**
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Write the failing app regression test**

```rust
#[tokio::test]
async fn verified_authorization_callback_marks_payment_authorized_without_fulfillment_side_effects() {}
```

- [ ] **Step 2: Write the failing HTTP regression test**

```rust
#[tokio::test]
async fn payment_callback_route_processes_authorization_without_capture() {}
```

- [ ] **Step 3: Write the failing portal order-center regression test**

```rust
#[tokio::test]
async fn portal_order_center_includes_authorized_payment_state() {}
```

- [ ] **Step 4: Run targeted tests to verify they fail**

Run: `cargo test --offline -p sdkwork-api-app-payment verified_authorization_callback_marks_payment_authorized_without_fulfillment_side_effects -- --nocapture`
Expected: FAIL because authorization callbacks are not normalized into a first-class state.

Run: `cargo test --offline -p sdkwork-api-interface-http payment_callback_route_processes_authorization_without_capture --test payment_callbacks -- --nocapture`
Expected: FAIL because the gateway callback route does not return `authorized`.

Run: `cargo test --offline -p sdkwork-api-interface-portal portal_order_center_includes_authorized_payment_state --test portal_order_center -- --nocapture`
Expected: FAIL because the order center cannot show the authorized payment lifecycle.

### Task 2: Implement authorization-aware payment state progression

**Files:**
- Modify: `crates/sdkwork-api-domain-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`

- [ ] **Step 1: Add `Authorized` state variants and `Authorization` transaction kind**
- [ ] **Step 2: Normalize authorization callbacks into an `authorized` outcome**
- [ ] **Step 3: Persist canonical authorization transaction evidence**
- [ ] **Step 4: Keep authorization from triggering fulfillment, account grants, or refund eligibility**
- [ ] **Step 5: Make state progression monotonic so late authorization callbacks cannot downgrade captured orders**

- [ ] **Step 6: Re-run the targeted app regression**

Run: `cargo test --offline -p sdkwork-api-app-payment verified_authorization_callback_marks_payment_authorized_without_fulfillment_side_effects -- --nocapture`
Expected: PASS

### Task 3: Expose authorization state through HTTP and portal surfaces

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Ensure HTTP callback responses serialize `authorized` normalized outcomes**
- [ ] **Step 2: Ensure portal order-center projections preserve `authorized` payment state and zero refundable amount**

- [ ] **Step 3: Re-run the targeted HTTP and portal regressions**

Run: `cargo test --offline -p sdkwork-api-interface-http payment_callback_route_processes_authorization_without_capture --test payment_callbacks -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-interface-portal portal_order_center_includes_authorized_payment_state --test portal_order_center -- --nocapture`
Expected: PASS

### Task 4: Re-verify payment lifecycle regressions

**Files:**
- Test: `crates/sdkwork-api-domain-payment/src/lib.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Test: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`
- Test: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Run domain payment tests**

Run: `cargo test --offline -p sdkwork-api-domain-payment -- --nocapture`
Expected: PASS

- [ ] **Step 2: Run payment app tests**

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS

- [ ] **Step 3: Run HTTP payment callback tests**

Run: `cargo test --offline -p sdkwork-api-interface-http --test payment_callbacks -- --nocapture`
Expected: PASS

- [ ] **Step 4: Run portal order-center tests**

Run: `cargo test --offline -p sdkwork-api-interface-portal --test portal_order_center -- --nocapture`
Expected: PASS
