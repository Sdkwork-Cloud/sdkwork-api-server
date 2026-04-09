# Portal Recharge Visual Overhaul Design

## Context

The current recharge page is functionally correct but visually reads like a stack of independent cards instead of a commercial purchase flow. It exposes recharge options, a quote surface, and history, but the hierarchy is weak:

- the page does not immediately explain the current balance posture
- preset cards feel repetitive rather than curated
- the quote panel looks like a secondary summary instead of the page's decision anchor
- history is visually detached from the selection and checkout path
- the page is acceptable as an internal tool, but not persuasive enough for a revenue-critical portal surface

The portal already has a strong framework and data path. This redesign should stay inside those boundaries and improve the product quality through layout, rhythm, emphasis, and copy density rather than through new backend dependencies.

## Goal

Turn recharge into a premium decision workspace that feels commercially intentional, credible, and fast to act on.

Success means:

- the first screen tells the user what state they are in and what action to take
- preset selection feels curated and comparable
- the quote card feels like the primary conversion surface
- the page remains compact, testable, and aligned with existing portal APIs
- mobile and desktop both preserve a clear top-to-bottom decision flow

## Non-Goals

- no backend or repository contract changes
- no new commerce entities
- no billing workbench merge
- no decorative-only sections that dilute the path to create order
- no large theme-system rewrite

## Approaches Considered

### 1. Luxury Checkout

Use a dramatic hero and immersive checkout composition.

Pros:

- strongest visual impact
- high conversion energy

Cons:

- too brand-led for the rest of the portal
- risks feeling disconnected from the control-plane product language

### 2. Finance Dashboard

Lean into analytical cards and before/after balance framing.

Pros:

- consistent with enterprise tooling
- low implementation risk

Cons:

- likely to remain dry and operational
- insufficient visual uplift for a revenue page

### 3. Decision Studio

Present recharge as a guided decision flow: current posture, curated amount selection, live payment outcome, then history.

Pros:

- preserves trust and operator clarity
- creates a clear revenue journey
- fits the existing portal shell and framework

Cons:

- requires careful visual hierarchy tuning to avoid becoming another dashboard

Chosen approach: `Decision Studio`

## Information Architecture

The page remains a three-section surface, but each section gets a sharper job:

### 1. Selection Studio

The left primary panel becomes a structured selection surface with:

- a compact posture header explaining remaining balance and purchase intent
- a curated preset matrix with stronger price typography and comparison chips
- a custom amount tile that feels first-class, not an afterthought

This section answers: "What should I buy?"

### 2. Quote Cockpit

The right sticky panel becomes the visual anchor:

- stronger amount hero
- balance delta and granted units surfaced as decision metrics
- tighter supporting copy
- a more assertive primary CTA

This section answers: "What happens if I buy it now?"

### 3. Recharge Timeline

The history surface becomes calmer and more legible:

- clearer header framing pending orders as follow-up actions
- cleaner separation between status, amount, and units
- footer controls that feel lighter and less table-like

This section answers: "What already happened and what still needs action?"

## Visual Direction

The page should feel like a premium financial instrument panel, not a generic SaaS form.

### Tone

- editorial, deliberate, high-trust
- bright but not washed out
- premium without luxury theatrics

### Layout

- stronger top hero inside the selection panel
- better use of asymmetric width between selection and quote
- more intentional negative space between content clusters
- visual rails that lead the eye from status to choice to action

### Typography

- larger money figures
- tighter tracking on critical numerics
- smaller and cleaner supporting labels
- remove filler copy and repeated explanations

### Surfaces

- soften card repetition by mixing glassy sub-panels with contrast blocks
- introduce subtle inset framing and atmosphere glows
- make the active state obviously premium, not merely selected

### Motion

- rely on existing transition classes for subtle elevation and emphasis
- avoid adding animation frameworks or heavy choreography

## Component-Level Changes

### Selection header

Add a compact posture banner inside the main selection card to expose:

- current balance
- recommended purchase signal
- number of pending orders

This provides decision context before the user touches the matrix.

### Preset cards

Refine preset cards to show:

- amount
- recommendation marker
- granted units
- effective ratio
- compact note slot or status line for selected/recommended states

Recommended cards should feel curated. Selected cards should feel locked and high-confidence.

### Custom amount tile

Custom input needs to feel like part of the same buying system:

- clearer title and support line
- better relationship between input and preview button
- min/step/max chips treated as product constraints, not raw validation noise

### Quote panel

Quote panel should include:

- amount hero
- pricing rule badge
- granted units
- effective ratio
- projected balance
- one short trust note above the CTA

The CTA should feel like the page's destination.

### History section

Keep the table, but improve surrounding structure:

- tighter container styling
- more intentional header hierarchy
- pending payment badge framed as an operations continuation
- less visual weight on pagination chrome

## Copy Strategy

Use fewer words and better words.

- prefer short action-oriented labels
- replace generic descriptions with operator-relevant signals
- keep support text to one sentence per cluster when needed
- avoid marketing slogans

## Responsiveness

Desktop:

- maintain split layout
- sticky quote panel stays visible during option browsing

Tablet/mobile:

- posture header first
- options grid collapses cleanly
- quote panel follows immediately after selection
- history remains scannable without feeling like a data dump

## Testing Strategy

Update recharge tests before implementation to lock the new structure:

- selection studio hero and posture data slots
- quote cockpit framing slots
- refined CTA support copy and history framing
- preserve existing repository and service contract assertions

Run targeted portal recharge tests plus portal typecheck after implementation.

## Approval Note

The user explicitly requested autonomous iteration without review interruptions. This design is therefore treated as approved for implementation unless later overridden by the user.
