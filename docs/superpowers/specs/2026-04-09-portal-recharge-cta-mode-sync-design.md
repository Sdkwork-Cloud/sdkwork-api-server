# Portal Recharge CTA Mode Sync Design

**Status:** aligned to the local `sdkwork-api-router` repository on 2026-04-09

**Goal:** make the recharge page use one coherent CTA model during post-order handoff instead of mixing pre-order and post-order actions in the same purchase panel.

## Executive Decision

The page now has a valid post-order handoff state, but the CTA hierarchy is still inconsistent.

On desktop, the main CTA can still read like a pre-order action while the surrounding panel is already telling the user to continue in billing. That weakens the product signal.

The correct next step is:

- when the page is in purchase mode, the primary CTA remains order creation
- when the page is in post-order handoff mode, the primary CTA becomes billing continuation
- the user still gets an explicit way to leave handoff mode and start another order

## Product Intent

The page should have exactly one dominant instruction at a time.

That means:

- before order creation: buy
- after successful order creation: settle

The product should never ask the user to do both at once with equal weight.

## Scope

### In scope

- switch the desktop quote-card primary CTA to `Continue in billing` during post-order handoff
- add an explicit `Create another order` secondary action inside the handoff panel
- keep the mobile sticky CTA aligned with the same state model
- clear handoff state when `Create another order` is chosen
- update tests to encode the new CTA mode contract

### Out of scope

- backend changes
- payment flow changes
- removing the ability to create multiple recharge orders

## Acceptance Criteria

The work is complete when:

- desktop and mobile CTA logic use the same post-order handoff rule
- `Continue in billing` is the dominant action during handoff
- `Create another order` is available as the intentional exit path from handoff
- tests, typecheck, and build pass
