# Router Admin Product Redesign Design

## Goal

Turn `apps/sdkwork-router-admin` into a more professional admin product by separating the experience into two clear product modes:

- `Overview` as an operator cockpit
- all business routes as minimal workbenches

The target outcome is not "more UI", but a more credible control-plane product with faster scanning, clearer decisions, fewer distractions, and stronger visual hierarchy.

## Confirmed Direction

The user approved the recommended direction:

- `Overview` becomes a dashboard-like cockpit
- `Users`, `Tenants`, `Catalog`, `Traffic`, and `Operations` become streamlined workbenches

This is a product redesign pass, not a shell rewrite. The shell, theme, settings center, and sidebar alignment work already completed stays in place.

## Product Problems In The Current State

The current admin app is functionally richer than before, but it still feels less professional than `claw-studio` because the product surfaces are too even in visual weight.

Main issues:

- too many sections compete for attention on the same page
- explanatory cards stay visible even when they do not help immediate action
- pages often have more than one "main job"
- KPI strips are too broad and do not always drive a decision
- tables, guidance cards, and summary cards often sit at the same hierarchy level
- interaction patterns vary slightly page to page, which weakens product-system trust

## Product Model

### 1. Operator Cockpit

Only `Overview` should behave like a cockpit.

Its job is to answer:

1. Is the system healthy right now?
2. What is drifting or at risk?
3. Where is load, cost, or activity concentrating?
4. What should the operator do next?

`Overview` should not feel like a list of all available admin data. It should feel like a prioritized executive operations surface.

### 2. Minimal Workbench

Every other route should behave like a focused workbench.

Each workbench must:

- expose one primary task
- keep one dominant data surface
- preserve filters while the user works
- move state mutation into dialogs or confirmations
- reduce visual clutter that reads like documentation instead of tooling

## Page-Level Design

### Overview

Target structure:

- compact global status band
- prioritized risk queue
- one trend section
- one operator action section

Remove:

- broad KPI wallpaper
- duplicated leaderboard-style surfaces unless they support a decision
- descriptive filler cards

Keep:

- risk ranking
- live control-plane signals
- compact trend and hotspot evidence
- direct navigation into affected workbenches

### Users

Primary job:

- find an identity and take action

Target structure:

- 3 to 4 identity KPIs
- one control bar for search and filters
- one main identity table
- dialogs for create and edit

Reduce:

- separate primary visual treatment for operator and portal sections
- always-visible guidance blocks

### Tenants

Primary job:

- manage workspace ownership structure

Target structure:

- tenant, project, and key KPIs
- one control bar
- one main workspace registry view
- supporting state for keys as secondary detail, not equal-weight main content

Reduce:

- stacked explanation cards
- multiple same-weight registry blocks

### Catalog

Primary job:

- maintain routing mesh readiness

Target structure:

- coverage KPIs
- one provider-centric workbench surface
- filtered views or compact sections for channels, credentials, and models

Reduce:

- full-page parity between all catalog subtypes
- large instructional blocks

### Traffic

Primary job:

- investigate usage or routing anomalies

Target structure:

- compact metrics
- one persistent query/filter console
- one active result surface at a time
- auxiliary insights only when they help triage

Reduce:

- multiple equal-weight result regions open at once
- simultaneous usage and routing emphasis unless explicitly requested by filter mode

### Operations

Primary job:

- assess runtime posture and intervene only when needed

Target structure:

- runtime health KPIs
- page-top latest intervention result
- one main runtime table
- secondary provider-health support view

Reduce:

- standing guidance cards
- duplicate status framing that does not change action quality

## Shared Workbench Contract

Every business route should converge on the same structure:

1. compact page header
2. 3 to 4 decision-oriented KPIs
3. one control bar
4. one dominant surface
5. dialogs and confirmations for mutations

Rules:

- one page, one main job
- one dominant data surface
- at most one primary CTA per page
- row actions should stay short and predictable
- descriptive text should explain consequences, not narrate the whole feature

## Interaction Design Rules

- Filters must sit above the main surface and remain stable after dialogs close.
- Create and edit flows remain dialog-based.
- Delete confirmations must explain impact and blocking dependencies.
- Inline actions should stay limited; overflow behaviors can be introduced where rows have too many actions.
- Empty states should guide the user toward the primary action instead of presenting general documentation.

## Visual Design Rules

- The dominant surface must visually outweigh all supporting cards.
- KPI count must stay low and high-signal.
- Color is for state and emphasis, not decoration.
- Secondary notes should become hints, badges, inline metadata, or dialog copy wherever possible.
- White space and layout rhythm should make each page look intentionally edited, not automatically composed.

## Self-Review Standard

The redesign is not complete if any of the following remain true:

- a page still looks like several unrelated blocks stacked together
- a non-critical note card draws as much attention as the main data surface
- the user cannot identify the page's main task within a few seconds
- more than one section appears to be the page's true primary focus
- page-level filtering and actions differ unnecessarily between modules

## Success Criteria

This redesign pass is successful when:

- `Overview` reads like an operator cockpit
- business pages read like focused workbenches
- common patterns are visibly consistent across modules
- explanatory clutter is reduced without harming clarity
- the product feels closer to a mature operations console than a feature demonstration
