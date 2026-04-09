# Portal Recharge Post-Order Handoff Design

**Status:** aligned to the local `sdkwork-api-router` repository on 2026-04-09

**Goal:** make the recharge page respond more intelligently immediately after order creation so the user is guided into billing settlement instead of being left at a generic create-order state.

## Executive Decision

The recharge page now explains pending settlement well enough at the page level.

The next weakness is temporal rather than structural:

- the user creates an order
- the page confirms it in status copy
- but the main interaction surface still behaves almost the same as before the order was created

That is not the best product behavior.

The page should briefly shift modes after a successful order creation:

- confirm that the latest order is ready for payment
- elevate a `Continue in billing` handoff CTA
- keep the create-another-order path available, but no longer as the immediate dominant instruction

## Product Intent

Immediately after order creation, the user should feel a clear state change:

- the order exists
- the next action is payment completion
- billing is now the correct destination

This is a post-purchase handoff moment, not another pre-purchase decision moment.

## Scope

### In scope

- track the most recently created recharge order in the current page session
- show a dedicated post-order handoff panel when that new order is still pending payment
- surface `Continue in billing` as the immediate next-step CTA
- let the mobile sticky CTA adapt to the post-order handoff state
- clear the handoff when the user starts choosing a new package or custom amount
- update tests to lock the new UI contract

### Out of scope

- backend changes
- new payment-session logic
- changing order creation semantics
- removing the ability to create another order

## UI Contract

The page should add:

- `data-slot="portal-recharge-post-order-handoff"`

The page should include copy around:

- `Order ready for payment`
- `Continue in billing`

## Interaction Rules

### After a successful create-order action

If the most recently created order is still the latest pending-payment order, the page should:

- show a strong success-handoff panel in the quote area
- surface a billing CTA that reads like the next required action
- keep the existing next-step pending-settlement callout available as broader context

### When the user starts a new decision

If the user clicks another preset or previews a custom amount, the post-order handoff should clear.

That keeps the page honest:

- when finishing one order, billing is the right next step
- when starting a fresh order decision, purchase selection becomes the right next step again

## Acceptance Criteria

The work is complete when:

- the page enters a visible post-order handoff state after successful order creation
- `Continue in billing` becomes the clearest next action in that state
- mobile CTA behavior also reflects the handoff state
- changing the selection clears the post-order handoff
- tests, typecheck, and build pass
