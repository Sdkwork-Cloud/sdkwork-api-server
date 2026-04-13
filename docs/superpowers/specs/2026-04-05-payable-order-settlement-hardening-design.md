# Payable Order Settlement Hardening Design

## Scope

This tranche closes a critical security and transaction-integrity gap in the portal commerce flow:

- payable portal orders must not be fulfillable by a portal user without verified payment evidence
- checkout surfaces must stop advertising self-service manual settlement for payable orders
- verified payment callbacks must remain able to complete fulfillment through a trusted internal path

Out of scope:

- provider-specific gateway session creation for Stripe, WeChat Pay, and Alipay
- admin manual recovery workflows
- refund approval and reconciliation screens

## Problem

The current portal flow still exposes `settle_order` as a checkout action for payable orders and allows portal clients to call:

- `POST /portal/commerce/orders/{order_id}/settle`
- `POST /portal/commerce/orders/{order_id}/payment-events` with `event_type = "settled"`

For a paid recharge or subscription order, that allows quota or membership fulfillment without a verified payment callback. After the recent canonical payment integration, this is even more dangerous because the system now also projects a `captured` payment order off the fulfilled commerce status.

That is not a commercial-grade payment system. Manual payable settlement must be treated as a privileged recovery action, not a portal capability.

## Design

### 1. Split portal settlement authority

Keep two fulfillment paths:

- `settle_portal_commerce_order(...)`: portal-facing path, only valid for zero-payment fulfillment
- `settle_portal_commerce_order_from_verified_payment(...)`: trusted internal path used by payment callback processing

Both paths share the same settlement side-effect engine so quota, coupon, and membership remain idempotent. The difference is only the payable-order authorization guard.

### 2. Block payable self-settlement

For orders where external payment is required:

- `POST /portal/commerce/orders/{order_id}/settle` returns `409 Conflict`
- `POST /portal/commerce/orders/{order_id}/payment-events` with `settled` also returns `409 Conflict`

Error semantics:

- message should clearly state that payable orders require verified payment confirmation
- order status must remain `pending_payment`
- quota and membership must remain unchanged

### 3. Remove checkout UI affordance

For payable portal checkout sessions and payment checkout bridges:

- keep `provider_handoff`
- keep `cancel_order`
- remove `manual_settlement`

This prevents frontend or SDK consumers from treating manual settlement as a supported product path.

### 4. Preserve trusted callback fulfillment

Payment callback processing already verifies signatures and deduplicates events. After this change:

- verified settlement callbacks still transition the commerce order to `fulfilled`
- account grants and other side effects still execute once
- portal surfaces continue observing the fulfilled state after callback ingestion

## Testing strategy

Add or update regression coverage for:

- payable checkout sessions no longer exposing `settle_order`
- portal payable `POST /settle` rejecting self-settlement and leaving billing unchanged
- verified payment callback still activating quota and membership
- payment checkout bridge projections no longer advertising manual settlement
