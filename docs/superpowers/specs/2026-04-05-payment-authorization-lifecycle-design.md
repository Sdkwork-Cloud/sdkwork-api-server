# Payment Authorization Lifecycle Design

## Scope

This tranche upgrades the payment model from "successful capture only" to a commercial-grade
authorization-aware lifecycle that can safely represent:

- customer checkout opened
- provider authorization succeeded but funds are not yet captured
- later capture settlement that finalizes fulfillment

The goal is:

- persist a first-class `authorized` payment state
- prevent authorization from triggering fulfillment or account grants
- allow a later verified settlement callback to advance the same payment order to capture
- expose the authorization state to gateway, portal, and order-center consumers

## Problem

The current payment model only understands four verified callback outcomes:

- settled
- failed
- canceled
- expired

That leaves a material commercial gap:

- providers such as Stripe manual-capture flows, card preauthorization flows, and some
  enterprise payment methods return an authorization signal before capture
- the system currently cannot represent "authorized but not captured"
- operators and portal users cannot distinguish "money hold exists" from "payment still
  waiting on customer"
- later capture callbacks have no intermediate state to advance from

Without authorization-aware state, the order center and transaction history are materially
weaker than a professional payment platform.

## Design

### 1. Add an authorization state across payment artifacts

Add `Authorized` to:

- `PaymentOrderStatus`
- `PaymentAttemptStatus`
- `PaymentSessionStatus`
- `PaymentCallbackNormalizedOutcome`

The payment order will also use a distinct fulfillment marker:

- `authorized_pending_capture`

Meaning:

- payment authorization is confirmed
- no delivery, quota grant, or financial settlement has happened yet

### 2. Add an authorization transaction record

Add `Authorization` to `PaymentTransactionKind`.

When a verified authorization callback is processed:

- persist one canonical authorization transaction for the payment order
- do not create finance journal entries
- do not grant account quota or ledger entries
- do not make the order refundable

This keeps the transaction history commercially useful without overstating settled revenue.

### 3. Normalize provider authorization callbacks

Treat provider events such as authorization success or "requires capture" as a first-class
normalized outcome.

Detection will use provider event type / status signals including tokens such as:

- `authorized`
- `requires_capture`
- `awaiting_capture`

The normalized outcome order must preserve the stronger terminal or settlement outcomes:

- settled still wins over authorization
- failed / canceled / expired must not be mistaken for authorization

### 4. Advance state monotonically

Authorization must be an intermediate state, not a downgrade risk.

Use ranked state reconciliation so that:

- `awaiting_customer -> authorized` upgrades correctly
- `authorized -> captured` upgrades correctly
- a late authorization callback cannot downgrade an already captured order
- fulfillment stays `fulfilled` once settlement completed

### 5. Surface authorization state to operators and users

Authorization state should appear consistently in:

- payment callback HTTP response `normalized_outcome`
- portal order center payment order status
- payment transaction history for the order
- checkout artifact repair flows that reconstruct missing attempt/session rows

## Testing strategy

Add regression coverage for:

1. verified authorization callback:
   - marks payment order / attempt / session as authorized
   - writes a single authorization transaction
   - does not fulfill commerce order
   - does not create account grants, ledger entries, or refund eligibility
2. later verified settlement after authorization:
   - upgrades the order to captured
   - fulfills once
   - keeps authorization transaction evidence and adds sale evidence
3. portal order center:
   - shows `authorized` payment status
   - shows `authorized_pending_capture`
   - keeps `refundable_amount_minor = 0`
4. HTTP callback route:
   - returns `normalized_outcome = authorized`

## Out of scope

- manual capture command APIs
- partial capture or multi-capture modeling
- authorization-expiry release workflows
- provider-side capture reconciliation queries
