# Refund Execution Kickoff Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an explicit refund execution kickoff step, tighten governed refund success finalization, and expose refund queue filtering for admin operations.

**Architecture:** Keep the current refund kernel intact, add a narrow `approved -> processing` transition in the payment app, expose it through a dedicated admin route, and filter admin refund listings in the HTTP layer by exact `refund_status`.

**Tech Stack:** Rust, axum, serde, sqlx-backed sqlite/postgres admin stores, cargo test

---

### Task 1: Lock refund execution kickoff behavior with failing tests

**Files:**
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`

- [ ] **Step 1: Add a failing payment-app test showing approved refunds cannot finalize until execution is started**
- [ ] **Step 2: Add a failing admin route test for starting refund execution**
- [ ] **Step 3: Add a failing admin refund queue test for `refund_status` filtering**
- [ ] **Step 4: Run focused tests and verify failure**

### Task 2: Implement execution kickoff and queue filtering

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`

- [ ] **Step 1: Add the payment-app helper to start approved refund execution**
- [ ] **Step 2: Tighten refund success finalization so approved portal-governed refunds must be started first**
- [ ] **Step 3: Add the admin start route and request DTO**
- [ ] **Step 4: Add refund list query parsing and exact `refund_status` filtering**

### Task 3: Re-verify the refund execution slice

**Files:**
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`

- [ ] **Step 1: Format touched Rust files**
- [ ] **Step 2: Re-run focused payment/admin tests**
- [ ] **Step 3: Re-run wider package regressions for payment, admin, and portal**
