# Portal Canonical Payment Checkout Design

## Scope

This tranche closes the next payment gap in the portal commerce flow:

- every paid portal commerce order must prepare canonical payment artifacts automatically
- portal account history must stop inferring canonical ownership from existing payment orders
- the portal order-center read model must not expose pending checkout orders as refundable

Out of scope:

- live Stripe, Alipay, and WeChat Pay gateway adapters
- refund approval workflow
- subscription proration and coupon restoration policies

## Problem

The current portal order flow can create a paid commerce order without also creating the canonical `payment_order`, `payment_attempt`, and `payment_session` records. That breaks the intended money flow in three ways:

- refund and order-center surfaces only become complete after tests or operators seed canonical payment records manually
- account-history currently derives canonical ownership from existing payment orders, which means it has no first-class portal identity bridge
- pending payment orders can appear structurally refundable in the read model even though the payment kernel correctly blocks refunds before capture

## Design

### 1. Stable portal-to-canonical subject bridge

Add a payment-side helper that resolves a portal user into a stable `PaymentSubjectScope`.

Behavior:

- look up an identity binding for binding type `portal_user`, issuer `sdkwork-portal`, subject `<portal_user_id>`
- if the binding exists, use its `tenant_id`, `organization_id`, and `user_id`
- if the binding does not exist, create a deterministic canonical tenant/user mapping, upsert the canonical identity user record, and persist the binding for future reuse

This keeps the portal identity bridge explicit and auditable without introducing a separate provisioning workflow.

### 2. Automatic canonical payment preparation

Add a payment helper that:

- accepts the current paid commerce order plus portal user id
- resolves the canonical payment subject through the bridge above
- calls `ensure_commerce_payment_checkout(...)`

Wire this helper in the portal interface at two points:

- immediately after creating a paid order
- when loading `/portal/commerce/orders/{order_id}/checkout-session` as an idempotent backfill path

This keeps existing portal commerce contracts intact while guaranteeing that normal checkout traffic prepares the canonical payment rail.

### 3. Account-history ownership

Change portal account-history loading to use the portal identity bridge instead of deriving ownership from the most recent payment order.

Behavior:

- resolve the canonical scope for the current portal user
- load the canonical account for that owner
- continue returning an empty payload when the account does not exist yet
- still list refund records only for payment orders belonging to the current project

### 4. Refundability semantics

Change the order-center read model so `refundable_amount_minor` is only non-zero when the payment order status actually supports refunds.

That keeps the portal read surface aligned with the payment kernel’s enforcement rules.

## Testing strategy

Add regression coverage for:

- paid portal order creation automatically preparing canonical payment artifacts
- portal order-center exposing an awaiting-customer payment order with zero refundable amount before settlement
- checkout-session retrieval remaining idempotent and not duplicating canonical payment artifacts
- portal account-history continuing to work after settlement/refund using the identity bridge
