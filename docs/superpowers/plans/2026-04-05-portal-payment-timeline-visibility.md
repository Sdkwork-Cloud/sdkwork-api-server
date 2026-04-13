# Portal Payment Timeline Visibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make payment failover attempts, active sessions, and callback history visible in the portal order center.

**Architecture:** Extend the canonical payment read model in `sdkwork-api-app-payment`, map the richer structure in the portal interface, and add a read-only GET handler on the existing payment-events route for history inspection.

**Tech Stack:** Rust, axum, serde, sqlx-backed stores, cargo test

---

### Task 1: Lock portal payment timeline behavior with failing tests

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Add a failing test that order-center exposes failover attempts and the active retry session**
- [ ] **Step 2: Add a failing test that `GET /portal/commerce/orders/{order_id}/payment-events` returns callback history**
- [ ] **Step 3: Run the focused portal order-center test target and verify failure**

Run: `cargo test --offline -p sdkwork-api-interface-portal --test portal_order_center -- --nocapture`
Expected: FAIL because the richer payment timeline fields and event history route do not exist yet.

### Task 2: Implement portal payment timeline read models and handlers

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`

- [ ] **Step 1: Add payment attempt trace structs and callback history loader helpers**
- [ ] **Step 2: Extend order-center entry assembly with payment attempts and active session selection**
- [ ] **Step 3: Extend portal response DTO mapping for the new order-center fields**
- [ ] **Step 4: Add GET handler for `/portal/commerce/orders/{order_id}/payment-events`**
- [ ] **Step 5: Re-run the focused portal order-center tests**

Run: `cargo test --offline -p sdkwork-api-interface-portal --test portal_order_center -- --nocapture`
Expected: PASS

### Task 3: Re-verify the affected slice

**Files:**
- Test: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`

- [ ] **Step 1: Re-run portal order-center tests**

Run: `cargo test --offline -p sdkwork-api-interface-portal --test portal_order_center -- --nocapture`
Expected: PASS

- [ ] **Step 2: Re-run payment callback processing tests**

Run: `cargo test --offline -p sdkwork-api-app-payment --test payment_callback_processing -- --nocapture`
Expected: PASS

- [ ] **Step 3: Re-run the portal package and payment package**

Run: `cargo test --offline -p sdkwork-api-interface-portal -- --nocapture`
Expected: PASS

Run: `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`
Expected: PASS
