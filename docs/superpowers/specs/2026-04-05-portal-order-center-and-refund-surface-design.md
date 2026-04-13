# Portal Order Center And Refund Surface Design

## Scope

This tranche exposes the already-landed canonical payment and refund kernel through portal and admin interfaces without rewriting the current portal checkout flow.

In scope:

- add a portal order-center read model that joins commerce orders with payment orders, payment transactions, and refund orders
- add a portal refund-request endpoint for supported paid recharge orders
- add a portal account-history endpoint for account balance, lots, ledger entries, and refund evidence
- add admin read endpoints for payment orders and refund orders so backoffice can inspect money movement

Out of scope:

- replacing the existing portal `checkout-session` bridge with canonical payment-order creation
- provider refund adapters and callbacks for Stripe, WeChat Pay, or Alipay
- subscription rollback and proration
- refund approval workflow and reconciliation screens

## Problem

The workspace now has canonical payment-order, payment-transaction, refund-order, finance-journal, quota-reversal, and account-ledger behavior underneath. That core is not yet visible to the portal or admin surface.

Current commercial gaps:

- a portal user cannot see refund state per order
- a portal user cannot request a refund through the product surface
- account grant and refund reversals exist in storage, but there is no portal history endpoint for them
- admin cannot inspect payment orders and refund orders through the HTTP control plane

That leaves the order center operationally incomplete even though the kernel exists.

## Design

### 1. Portal order-center read model

Add a new portal endpoint:

- `GET /portal/commerce/order-center`

Response shape:

- one entry per commerce order, sorted by `created_at_ms desc`
- nested `payment_order`
- nested `payment_transactions`
- nested `refunds`
- computed `refundable_amount_minor`

This endpoint must not change the existing `/portal/commerce/orders` contract. Existing consumers keep their current lightweight order list. New order-center consumers opt into the richer read model.

### 2. Portal refund request

Add a new portal endpoint:

- `POST /portal/commerce/orders/{order_id}/refunds`

Request body:

- `refund_reason_code`
- `requested_amount_minor`

Behavior:

- load the current portal workspace and verify the order belongs to the authenticated user and project
- find the canonical payment order bound to the commerce order
- reject when no payment order exists
- derive `PaymentSubjectScope` from the matched payment order record
- call `request_payment_order_refund(...)`
- return the created refund order

This keeps refund authorization anchored to existing order ownership plus kernel scope checks, instead of inventing a second refund permission model.

### 3. Portal account history

Add a new portal endpoint:

- `GET /portal/billing/account-history`

Response shape:

- optional `account`
- optional `balance`
- `lots`
- `ledger_entries`
- `ledger_allocations`
- `refunds`

Scope derivation:

- infer the canonical account owner from the current workspace payment orders
- if no canonical payment order exists yet, return an empty history payload instead of failing

This is intentionally pragmatic. The current portal identity model is string-based, while the account kernel is numeric-subject based. Until the portal checkout flow is fully bridged into canonical identity and payment creation, the account-history surface will use payment-order ownership as the canonical join point.

### 4. Admin payment inspection

Add new admin endpoints:

- `GET /admin/payments/orders`
- `GET /admin/payments/refunds`

These are read-only list endpoints for operational inspection. They intentionally avoid approve/finalize mutations in this tranche because provider refund orchestration and approval workflow are not fully designed yet.

### 5. Data access strategy

Reuse existing store primitives:

- `list_commerce_orders_for_project`
- `list_payment_order_records`
- `list_payment_transaction_records_for_order`
- `list_refund_order_records_for_payment_order`
- `find_account_record_by_owner`
- `list_account_benefit_lots`
- `list_account_ledger_entry_records`
- `list_account_ledger_allocations`

Portal order-center queries are user-scoped and expected to have small cardinality. For this tranche, per-order transaction/refund fan-out is acceptable. If this endpoint later becomes a high-volume admin or cross-tenant report, add batched list queries in the storage trait rather than embedding ad hoc SQL in the interface layer.

## Error handling

Portal refund request must return:

- `404` when the order or payment order is not visible in the current workspace
- `400` for invalid refund inputs
- `409` only when the underlying kernel surfaces a business conflict that maps cleanly to conflict semantics later; this tranche can continue using the current commerce-style error mapping if needed

Portal account history must return:

- `200` with empty arrays and `null` account/balance when no canonical payment/account data exists yet

## Testing strategy

Add interface-first TDD coverage for:

- order-center includes payment and refund state for a paid recharge order
- portal refund request creates a refund order for an owned paid recharge order
- portal account-history shows grant and refund ledger evidence after refund success
- admin payment-order and refund-order endpoints expose seeded records

## Known follow-ups

- bridge portal checkout into canonical payment-order creation so every paid order always has a payment record
- create a canonical portal-to-identity subject mapping instead of deriving account ownership from payment orders
- add provider refund callbacks, approval workflow, reconciliation, and alerting
