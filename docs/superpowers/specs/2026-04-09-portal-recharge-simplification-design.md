# Portal Recharge Simplification Design

**Status:** aligned to the local `sdkwork-api-router` repository on 2026-04-09

**Goal:** simplify the portal recharge page so it behaves like a commercial purchase surface instead of an analysis-heavy operations page, while preserving the existing recharge option, quote, order-creation, and history capabilities.

## Executive Decision

The portal recharge page should stop behaving like a finance analysis dashboard.

The correct target experience is a focused three-section recharge surface:

- recharge options
- payment information
- recharge history

The page should keep custom recharge amounts, but the entire visual hierarchy should be reorganized around conversion and purchase confidence instead of summary metrics and decision-support analytics.

This means:

- remove the summary grid
- remove the decision-support section
- keep the existing recharge option and quote logic
- redesign the recharge option and quote presentation into a commercial SaaS purchase flow
- keep recharge history as a lower-priority operational section

## Verified Current Baseline

The existing portal recharge page already has working business behavior, but the presentation is too dense for the intended purchase task.

### Current page structure

The current implementation in `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx` renders:

- `portal-recharge-summary-grid`
- `portal-recharge-decision-support`
- `portal-recharge-options`
- `portal-recharge-custom-form`
- `portal-recharge-quote-card`
- `portal-recharge-history-table`

### Current business behavior worth preserving

The page already supports the correct recharge workflow:

- server-managed recharge options
- custom recharge amount preview and validation
- quote preview
- recharge order creation
- recharge order history
- a secondary navigation path back to billing

These behaviors are backed by the existing repository and API seams and should remain intact.

### Current product problem

The page currently asks the user to process too much analytical context before acting:

- top-level summary cards consume prime visual space
- finance projection and multimodal demand cards add operational context that is not required for buying recharge
- the recharge option area competes with other sections instead of acting as the primary decision surface
- the quote card reads as a tooling panel instead of a payment confirmation panel

The result is a page that is informative but not commercially effective.

## Product Intent

The recharge page should feel like a premium SaaS purchase flow inside the portal:

- professional
- trustworthy
- commercially polished
- conversion-oriented without becoming loud or cheap

The desired emotional outcome is:

- the user immediately sees what they can buy
- the recommended option feels safe and attractive
- the payment information area feels like a clear confirmation step
- history remains available without distracting from the current purchase

## Scope

### In scope

- simplify the recharge page to three main sections
- keep preset recharge options
- keep custom recharge amount entry
- keep payment information and quote-driven order creation
- keep recharge history
- redesign the visual hierarchy to feel more commercial and productized
- update page-level tests so they encode the simplified contract

### Out of scope

- backend API changes
- order creation semantics
- recharge pricing policy behavior
- billing page redesign
- new payment methods or new payment flow logic
- removing custom recharge

## Target Page Structure

The final recharge page should have only three primary sections.

### 1. Recharge options

This is the main decision surface.

It should include:

- preset recharge cards
- a clearly highlighted recommended option
- the custom recharge input as an advanced but still first-class path

It should not feel like a technical form. It should feel like a set of purchase packages.

### 2. Payment information

This replaces the current “create recharge order” tooling feel with a cleaner commercial confirmation panel.

It should show:

- selected recharge amount
- granted units
- effective ratio
- projected balance
- current balance
- the primary purchase action

This section should become the page’s secondary visual anchor after the option cards.

### 3. Recharge history

This remains available for operator confidence and auditability, but it should be visually subordinate to the purchase flow.

It should keep:

- order history table
- status badges
- pending-payment visibility
- secondary navigation to billing

## Visual Direction

The chosen direction is a high-trust commercial SaaS purchase surface.

This is intentionally not:

- a discount-heavy ecommerce promotion page
- a neutral internal operations dashboard

It should balance:

- strong conversion cues
- clear pricing hierarchy
- enterprise trust
- consistency with the existing portal visual system

### Recharge option cards

The option cards should become the strongest visual element on the page.

Required characteristics:

- larger, more deliberate card treatment
- stronger selected state through border, background, and elevation
- visible recommended badge on the preferred option
- amount, granted units, and ratio clearly separated into primary and secondary hierarchy
- cleaner whitespace and more premium spacing

The recommended option should feel intentionally merchandised, not merely toggled.

### Payment information panel

The payment information panel should read like a purchase confirmation component, not a utility panel.

Required characteristics:

- clear settlement summary layout
- large emphasis on selected amount
- supporting purchase facts rendered as clean invoice-like rows
- one unambiguous primary CTA
- copy that reinforces readiness and clarity instead of analytical explanation

### Recharge history section

Recharge history should keep a simpler visual treatment:

- standard card shell
- cleaner heading and supporting copy
- less decorative emphasis than the purchase area
- preserve usability and status readability

## Interaction Design

### Default purchase path

The default flow should privilege a quick purchase decision:

- load the recommended preset option as the default selected state
- immediately show a populated payment information panel when possible
- allow one-click movement between recharge cards and quote updates

### Custom recharge path

Custom recharge should remain supported, but it should not overpower the default package-selection flow.

Design rules:

- keep it inside the recharge options section
- place it after the preset cards
- style it as an advanced purchase entry instead of a loose utility form
- preserve the same quote and validation behavior as today

### CTA behavior

There should be one clear primary action:

- `Create recharge order`

The billing navigation link remains secondary and belongs in the history area, not in the main purchase funnel.

## Data and Logic Boundaries

The redesign should not change the underlying recharge logic.

The implementation should continue to use the current:

- `loadPortalRechargePageData`
- `previewPortalRechargeQuote`
- `createPortalRechargeOrder`
- `buildPortalRechargeHistoryRows`
- recharge validation helpers

The page may remove rendering paths and derived UI copy that only existed to support the deleted summary and decision-support sections.

## Test Strategy

The redesign should be implemented test-first at the page-contract level.

### New page contract

The page must keep:

- `data-slot="portal-recharge-page"`
- `data-slot="portal-recharge-options"`
- `data-slot="portal-recharge-quote-card"`
- `data-slot="portal-recharge-history-table"`

The page must remove:

- `data-slot="portal-recharge-summary-grid"`
- `data-slot="portal-recharge-decision-support"`
- `data-slot="portal-recharge-multimodal-demand"`

### Behavioral expectations that remain true

- custom recharge validation still works
- recharge options still drive quote generation
- quote-driven order creation still works
- recharge history still exposes the billing workbench secondary path

### Visual-content expectations

Tests should verify the simplified copy contract:

- keep `Recharge options`
- keep `Recharge history`
- keep `Create recharge order`
- remove old decision-support copy expectations

## Acceptance Criteria

The work is complete when all of the following are true:

- the recharge page only presents recharge options, payment information, and recharge history as primary sections
- summary-grid and decision-support sections are removed
- custom recharge remains available
- the page looks commercially stronger and more purchase-oriented than the current workspace-style layout
- existing recharge business behavior is preserved
- portal recharge page tests are updated to the simplified contract and pass

## Implementation Notes

The most important implementation discipline is to simplify without hollowing the page out.

The result should not look like a partially deleted dashboard. It should look like an intentional commercial purchase page that belongs in the portal product.
