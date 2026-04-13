# Payment Attempt Failover Design

## Scope

This tranche activates the existing payment gateway account and payment channel policy tables so
the payment service can recover from terminal checkout failures by switching to the next eligible
route.

The scope is intentionally narrow:

- add domain and storage support for gateway accounts and channel policies
- let failed or expired payment attempts create a replacement attempt and session
- keep the payment order open when an alternate route exists

## Problem

The current payment flow detects callback outcomes correctly, but terminal non-settlement outcomes
still behave as a dead end:

- `failed` and `expired` callbacks close the payment order immediately
- no route selection happens even though gateway and channel policy tables already exist
- operators cannot rely on the core payment service to attempt the next configured route

This leaves the order center below commercial expectations for failure recovery.

## Approaches Considered

### Approach A: Keep terminal failures final and rely on manual restart

Pros:

- no new orchestration logic

Cons:

- poor recovery experience
- wastes the existing routing schema
- leaves checkout recovery fully manual

### Approach B: Add policy-driven failover inside the payment service

Pros:

- activates the existing schema with a minimal change set
- keeps recovery close to payment state transitions
- preserves a full attempt/session audit trail

Cons:

- only handles simple deterministic failover
- still lacks rich health scoring or weighted routing

### Approach C: Introduce a separate payment routing engine first

Pros:

- strongest long-term architecture

Cons:

- far too large for the current hardening slice
- would delay the first commercially meaningful recovery behavior

Recommended: Approach B.

## Design

### 1. Add canonical domain and store models for payment routing configuration

Add records for:

- `PaymentGatewayAccountRecord`
- `PaymentChannelPolicyRecord`

The storage layer gains insert/list methods so tests and future admin management surfaces can
persist and read these rows consistently across sqlite and postgres.

### 2. Define deterministic route selection for failover

When a payment attempt ends in a recoverable terminal outcome, the payment service selects the
next route by:

1. loading active channel policies for the order scope
2. filtering policies by:
   - tenant / organization
   - `scene_code` empty or equal to `payment_order.order_kind`
   - `currency_code` empty or equal to `payment_order.currency_code`
   - `client_kind` empty or equal to the current attempt client kind
3. ordering policies by `priority DESC`, then `channel_policy_id ASC`
4. finding active gateway accounts for the policy provider ordered by `priority DESC`, then
   `gateway_account_id ASC`
5. excluding gateway accounts already used by prior attempts for the same payment order

The selected route uses:

- provider from the policy
- method from the policy
- gateway account from the matching account row

### 3. Only `failed` and `expired` outcomes trigger automatic failover

Automatic failover applies when:

- the callback outcome is `failed` or `expired`
- the payment order has not already reached a captured/refundable state
- an alternate eligible route exists

`canceled` remains terminal in this tranche because it is often user-driven and should not reopen
checkout automatically.

### 4. Failover creates a new attempt/session and reopens the order

When failover is triggered:

- the current attempt remains terminal (`failed` or `expired`)
- the current session remains terminal
- a new payment attempt is inserted with:
  - incremented `attempt_no`
  - selected gateway account / provider / method
  - `attempt_status = handoff_ready`
- a new payment session is inserted with:
  - `session_status = open`
  - a fresh attempt-linked session id
- the payment order is reopened to:
  - `payment_status = awaiting_customer`
  - `fulfillment_status = pending`
  - selected provider / method on the order

The processed callback result returns the replacement attempt and session so callers can surface
the active next step immediately.

### 5. No failover means existing terminal behavior stays unchanged

If no alternate route exists, the current behavior remains:

- order stays `failed` or `expired`
- no replacement attempt/session is created

## Testing Strategy

Add regression coverage for:

1. sqlite/postgres round-trip of gateway accounts and channel policies
2. failed callback with configured alternate route:
   - original attempt/session become terminal
   - replacement attempt/session are created
   - payment order returns to `awaiting_customer`
3. expired callback with configured alternate route:
   - same reopen behavior
4. canceled callback:
   - remains terminal
   - no automatic failover
5. duplicate failed callback:
   - does not create an extra replacement attempt

## Out Of Scope

- admin CRUD for gateway accounts and channel policies
- geographic matching beyond current available request/order context
- weighted routing, health scoring, or circuit breaking
- provider-native retry orchestration
