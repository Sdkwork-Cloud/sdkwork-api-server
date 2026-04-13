# Pending Order Reuse Design

## Scope

This tranche hardens the payable order creation path against duplicate pending orders caused by:

- client retries after timeouts or network loss
- repeated clicks on the same checkout CTA
- prior create attempts that persisted the commerce order but did not complete checkout preparation cleanly

Out of scope:

- explicit client-supplied idempotency keys
- provider-native payment intent reservation
- background order deduplication or historical merge jobs

## Problem

The current portal create flow always inserts a new payable commerce order after quote evaluation. If the same request is retried:

- the project can accumulate multiple `pending_payment` orders for the same intended purchase
- each duplicate order gets its own canonical payment artifacts
- order center and portal UX become ambiguous about which order should actually be paid

That is not commercial-grade behavior for a checkout system. A pending payable order should act like an open checkout intent and be reused while it is still valid.

## Options Considered

### Option 1: Require a client idempotency key now

Pros:

- strongest long-term contract
- explicit and auditable retry semantics

Cons:

- requires API contract and client changes
- does not help existing callers immediately

### Option 2: Server-side pending-order reuse

Pros:

- backward compatible
- immediately stops duplicate pending orders for repeated identical purchase attempts
- composes naturally with the new checkout artifact recovery logic

Cons:

- matching is heuristic and must be carefully scoped
- not a full replacement for explicit idempotency keys

### Option 3: Full transactional create plus payment preparation

Pros:

- strongest consistency boundary

Cons:

- larger architectural slice
- needs broader storage and failure-semantics work

## Recommendation

Implement Option 2 now.

For payable orders only, before creating a new commerce order:

- compute the canonical quote
- search project-scoped orders for the same user and same economic purchase intent
- if a matching `pending_payment` order already exists, return that order instead of creating a new one

Matching should require:

- same project
- same user
- same `target_kind`
- same `target_id`
- same `payable_price_cents`
- same granted / bonus unit outcome
- same applied coupon identity
- status still `pending_payment`

This keeps reuse narrow and deterministic enough for commercial checkout behavior.

## Design

### 1. Reuse only payable orders

Zero-payment fulfillment flows such as coupon redemption are excluded. They should continue using their existing direct-fulfillment path.

### 2. Reuse only the latest equivalent pending order

When multiple historical orders exist, pick the newest matching `pending_payment` record. Older canceled, failed, or fulfilled orders must not be reused.

### 3. Let existing checkout recovery repair artifacts

The create handler already runs checkout synchronization after `submit_portal_commerce_order(...)` returns. Once the pending order is reused:

- missing payment artifacts can be repaired automatically
- no extra handler-level branching is required

## Testing strategy

Add or update regression coverage for:

- direct commerce submission reusing the same pending payable order on repeated identical requests
- portal `POST /portal/commerce/orders` returning the same order id for repeated identical payable create requests
- order center and canonical payment artifacts remaining singular after repeated create requests
