# Refund Approval Governance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an approval gate for portal-originated refunds and expose admin actions to approve or cancel refund requests before execution.

**Architecture:** Keep the low-level refund kernel intact, but move the portal submission path into `awaiting_approval`, add explicit admin actions for approval/cancelation, and tighten refund success finalization so unapproved portal refunds cannot complete.

**Tech Stack:** Rust, axum, serde, sqlx-backed sqlite/postgres admin stores, cargo test

---

### Task 1: Lock governance behavior with failing tests

**Files:**
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`

- [ ] **Step 1: Add a failing payment-app test showing awaiting-approval refunds cannot be finalized**
- [ ] **Step 2: Add a failing admin test for refund approve and cancel routes**
- [ ] **Step 3: Update portal refund tests to expect `awaiting_approval`**
- [ ] **Step 4: Run focused tests and verify failure**

### Task 2: Implement refund approval actions and portal awaiting-approval submission

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`

- [ ] **Step 1: Add payment-app helpers to approve and cancel refund requests**
- [ ] **Step 2: Transition portal refund submission into `awaiting_approval`**
- [ ] **Step 3: Tighten refund finalization to reject unapproved refund requests**
- [ ] **Step 4: Add admin routes and request DTOs for approve/cancel**

### Task 3: Re-verify the payment/admin/portal refund slice

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_payments.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_order_center.rs`
- Modify: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`

- [ ] **Step 1: Format touched Rust files**
- [ ] **Step 2: Re-run focused payment/admin/portal tests**
- [ ] **Step 3: Re-run wider package regressions**
