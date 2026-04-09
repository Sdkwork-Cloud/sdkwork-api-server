# Portal Recharge Settlement Continuity Design

**Status:** aligned to the local `sdkwork-api-router` repository on 2026-04-09

**Goal:** make the portal recharge page feel complete after order creation by surfacing the pending-payment next step near the purchase flow instead of leaving settlement guidance buried in history.

## Executive Decision

The recharge page now sells well enough, but the post-order path is still too soft.

Today the user can:

- pick a recharge amount
- review the quote
- create the order

But the next operational action still lives mostly in the lower history section. That means the page gets the user to a created order, then weakens the handoff to payment completion.

This iteration should not add new payment logic. It should make the existing handoff more explicit.

The chosen solution is to add a pending-settlement callout close to the checkout summary that:

- appears when pending-payment orders exist
- highlights the latest pending order
- reinforces that billing is the next step
- keeps the history table as the deeper audit surface

## Product Intent

The recharge page should feel like a complete buying loop, not a partial order-creation step.

The user should finish this page with a clear understanding of:

- whether there is still an unfinished payment
- which pending order is the freshest one
- where to go next to complete settlement

The page should say, in product terms:

- decision made
- order created
- payment follow-up lives in billing

## Scope

### In scope

- derive a lightweight pending-settlement spotlight from existing order history
- render a visible next-step callout near the checkout summary
- show latest pending order amount and recorded time
- add a stronger billing handoff CTA in the callout
- update page tests to lock the new contract

### Out of scope

- payment provider integration changes
- new checkout session behavior
- changing order status logic
- replacing the history table
- backend API changes

## Recommended Placement

The callout should live inside the right-side purchase zone, below the quote breakdown and above the historical audit section.

This placement is intentional:

- the user sees the next-step guidance while still in the purchase context
- the existing history table remains available for detail, but not responsible for primary guidance
- mobile layouts still keep the handoff visible because the right-side card collapses into the main flow

## Data and Logic Boundaries

This work should reuse the current order list already loaded by `loadPortalRechargePageData`.

No new API calls are needed.

The page should derive a pending-settlement spotlight from the existing order set:

- filter for `pending_payment`
- sort by newest first
- use the latest record as the spotlight anchor

The helper should live in the recharge service layer rather than inline in the page so the page file does not keep absorbing more business-adjacent view logic.

## UI Contract

The page should add:

- `data-slot="portal-recharge-next-step-callout"`

The callout should include product copy around:

- `Pending settlement queue`
- `Latest pending order`
- `Open billing to complete payment`

## Acceptance Criteria

The work is complete when:

- pending-payment orders are surfaced near the checkout area
- the latest pending order is visually highlighted as the next operational action
- the history table still remains intact as the audit surface
- no backend behavior changes are introduced
- portal recharge tests encode the new callout contract and pass
