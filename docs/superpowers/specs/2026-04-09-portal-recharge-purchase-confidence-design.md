# Portal Recharge Purchase Confidence Design

**Status:** aligned to the local `sdkwork-api-router` repository on 2026-04-09

**Goal:** strengthen the portal recharge page so the existing purchase flow feels more decisive, trustworthy, and mobile-resilient without changing backend behavior.

## Executive Decision

The recharge page already has the right information architecture.

The next step is not more sections or more analytics. The next step is better purchase confidence.

This iteration should keep the current three-part structure:

- recharge options
- payment information
- recharge history

But it should deepen the commercial quality of the experience in three targeted ways:

- give each purchase option a clearer intent and recommendation story
- make the payment information panel read like a real checkout confirmation surface
- preserve a strong primary action on mobile with a sticky action bar

## Verified Current Baseline

The current page in `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx` already:

- defaults into a recommended option
- previews quote information immediately
- supports custom amount validation and preview
- creates recharge orders
- exposes recharge history and a route back to billing

The current visual issue is subtler than the previous round.

The page is now clean, but parts of it still behave like good UI composition rather than a finished commercial purchase surface:

- preset cards look premium, but they do not explain the buying intent strongly enough
- the quote card is clear, but it still reads more like a structured status panel than a settlement confirmation
- on mobile, the primary CTA depends too much on the reader staying within the quote card viewport

## Product Intent

The recharge page should help a user make a faster, lower-friction decision with higher confidence.

The experience should answer three product questions immediately:

- which option is best for my current funding posture
- what exactly happens if I choose it
- where is the action I need to take right now

The page should feel like a premium B2B SaaS recharge workflow:

- commercially polished
- trustworthy
- decisive
- legible on desktop and mobile

## Scope

### In scope

- add stronger merchandising and guidance language to recharge options
- add a purchase narrative or decision strip that reinforces the currently selected path
- redesign the quote panel into a clearer checkout-style confirmation surface
- add a mobile sticky CTA summary when a selection is active
- keep all existing quote, order creation, and history behaviors
- update page-level tests to lock the new UI contract

### Out of scope

- backend API changes
- new payment methods
- checkout session integration
- billing page redesign
- changing order creation semantics
- adding promotions or coupon flows to recharge

## Target Experience

### 1. Recharge options become a guided package matrix

Each option card should communicate not only amount and value, but also intent.

The page should infer a simple commercial story from the existing catalog:

- the recommended option reads like the safest default
- lower amounts read like quick coverage or immediate runway
- higher amounts read like reserve-building or scale planning
- custom amount reads like precision control for operators who already know the number they need

The goal is not to fabricate financial analysis. The goal is to merchandise the same data more intelligently.

### 2. Payment information becomes a checkout confirmation panel

The right-side panel should stop reading like a cluster of metrics.

It should become a clear confirmation stack:

- selected amount as the anchor
- concise purchase reason or posture summary
- clean breakdown rows for units, projected balance, pricing rule, and current state
- one strong primary CTA
- one short settlement reassurance note

The panel should feel closer to invoice confirmation than dashboard telemetry.

### 3. Mobile keeps the action visible

On narrow screens, users should not lose the primary CTA after scrolling through options or history.

When a valid selection exists, a sticky bottom bar should remain available on mobile with:

- selected amount
- granted-units summary
- primary create-order CTA

This bar should be visually subordinate to desktop sticky behavior and should only appear when it materially helps.

## Data and Logic Boundaries

This iteration should stay inside current front-end seams.

The implementation should continue using:

- `loadPortalRechargePageData`
- `previewPortalRechargeQuote`
- `createPortalRechargeOrder`
- `buildPortalRechargeQuoteSnapshot`
- `buildPortalRechargeHistoryRows`
- `validatePortalRechargeAmount`

If new copy or intent labels are needed, derive them locally from existing recharge option ordering, recommendation flags, and quote snapshot values.

No new API calls should be introduced.

## Visual Direction

The visual direction is premium commercial software rather than ecommerce promotion.

Required characteristics:

- stronger hierarchy between the selected path and alternative paths
- more explicit reason-to-buy language
- cleaner invoice-style breakdown blocks
- clear mobile-safe action placement
- no noisy promotional gimmicks, discount theatrics, or fake urgency

## Test Strategy

This iteration should again be implemented test-first at the page-contract level.

### New UI contract expectations

The page should add:

- `data-slot="portal-recharge-guidance-band"`
- `data-slot="portal-recharge-selection-story"`
- `data-slot="portal-recharge-quote-breakdown"`
- `data-slot="portal-recharge-mobile-cta"`

The page should keep:

- `data-slot="portal-recharge-page"`
- `data-slot="portal-recharge-options"`
- `data-slot="portal-recharge-custom-form"`
- `data-slot="portal-recharge-quote-card"`
- `data-slot="portal-recharge-history-table"`

### Content expectations

Tests should confirm the refined purchase language stays present, including:

- `Best fit for steady usage`
- `Selection story`
- `Checkout summary`
- `Create order in billing`

Exact wording can be adjusted during implementation if the tests are updated intentionally and the product intent is preserved.

## Acceptance Criteria

The work is complete when all of the following are true:

- recharge options provide clearer decision guidance without adding backend complexity
- the quote card reads like a checkout confirmation surface, not a metrics panel
- a mobile sticky action summary exists for active selections
- existing recharge business behavior remains intact
- portal recharge tests encode the new UI contract and pass
