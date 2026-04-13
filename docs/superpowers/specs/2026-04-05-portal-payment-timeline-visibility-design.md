# Portal Payment Timeline Visibility Design

## Scope

This tranche makes payment recovery and payment history visible in the portal order center.

In scope:

- extend the portal order-center read model with payment attempt history
- expose the currently active payment session in the order-center response
- add `GET /portal/commerce/orders/{order_id}/payment-events` for callback-event history

Out of scope:

- changing payment write flows
- provider-native event enrichment
- timeline pagination
- admin timeline surfaces

## Problem

The payment kernel can now create replacement attempts and sessions after failed or expired
callbacks, but portal surfaces still hide that recovery path:

- order-center entries show payment orders, transactions, and refunds only
- the current active retry session is not visible to the user
- the payment-events route is mutation-only and cannot return the callback history

That leaves the order center below commercial expectations for transparency and supportability.

## Design

### 1. Extend the order-center read model

Add a payment attempt trace model:

- `attempt`
- `sessions`

Extend each order-center entry with:

- `payment_attempts`
- `active_payment_session`

Ordering:

- attempts by `attempt_no DESC`, then `payment_attempt_id DESC`
- sessions within an attempt by `created_at_ms DESC`, then `payment_session_id DESC`

`active_payment_session` is the first non-terminal or newest session from the latest active
attempt; otherwise it falls back to the newest session of the newest attempt.

### 2. Add callback-event history endpoint

Expose:

- `GET /portal/commerce/orders/{order_id}/payment-events`

Behavior:

- verify portal workspace ownership against the commerce order
- resolve the canonical payment order
- filter payment callback events by `payment_order_id`
- return newest-first records

The existing `POST /portal/commerce/orders/{order_id}/payment-events` remains unchanged for the
simulation/update flow already present in the portal API.

### 3. Data access

Reuse existing storage primitives:

- `list_payment_attempt_records_for_order`
- `list_payment_session_records_for_attempt`
- `list_payment_callback_event_records`

This slice accepts per-order fan-out because portal order-center usage is user-scoped and low
volume relative to admin reporting.

## Testing Strategy

Add portal interface coverage for:

1. order-center includes multiple payment attempts and the active replacement session after
   failover
2. payment-events GET returns newest-first callback records for the owned order
