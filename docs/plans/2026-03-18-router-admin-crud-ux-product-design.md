# Router Admin CRUD UX Product Design

**Date:** 2026-03-18

## Goal

Upgrade the admin management modules so they behave like a real backend workbench instead of mixed list-plus-form screens. Creation and editing move into focused modals, tables become easier to scan, and page structure aligns with the shell quality already established in the admin app.

## Problem

The current `Users`, `Tenants`, and `Coupons` modules still mix several jobs into the same page region:

- query and scan a registry
- create a new record
- edit an existing record
- review side-context such as workspace posture or campaign state

That creates three product issues:

- the primary list loses visual priority because forms consume the same space
- operators cannot quickly distinguish “I am browsing” from “I am mutating data”
- page density is high, but information hierarchy is weak

This is the opposite of the interaction model users expect from backend systems.

## Recommended Approach

Adopt a **workbench CRUD pattern** across admin modules.

Each management page should follow this structure:

1. `Hero` for purpose and quick actions
2. `KPI strip` for state awareness
3. `Toolbar` for search, filters, and entry actions
4. `Registry table` for scanning and row-level actions
5. `Context panel` for selected or active operational guidance

Mutation flows move out of the page body and into focused modals.

## Alternatives Considered

### 1. Keep inline forms and only restyle them

Not chosen because the structural problem remains. A prettier inline form still competes with the registry table.

### 2. Move all create and edit flows to separate full pages

Not chosen because these admin flows are relatively compact and benefit from staying close to the table context. Full pages would add navigation churn without enough payoff.

### 3. Use modals for create and edit, keep tables as the main page surface

Chosen because it best matches backend expectations:

- fast entry into mutations
- strong distinction between browse mode and edit mode
- no loss of table visibility
- easier reuse of form patterns across modules

## Product Rules

### CRUD layering

- List pages must not contain always-visible creation forms.
- Create and edit actions must open a modal or route to a dedicated editor surface.
- Tables remain the primary visual anchor on registry pages.

### Action hierarchy

- Primary CTA sits in the hero or toolbar.
- Row actions should stay concise: `Edit`, `Disable/Archive`, `Delete`, `Restore`.
- Secondary operational notes belong in a side panel or footer surface, not in the main action zone.

### Visual hierarchy

- Toolbars use compact controls in one horizontal rhythm on desktop.
- Modals use a stronger elevated surface than page cards.
- Page content should read top-to-bottom as: understand state, filter data, act on rows.

### Interaction quality

- Editing a row preloads modal fields.
- Closing a modal should leave the table state intact.
- Filters remain persistent while opening and closing modals.
- Destructive actions remain in-row, but creation and editing become focused flows.

## Module-Specific Design

### Users

Split entry points by population:

- `Create operator`
- `Create portal user`

Keep two registry surfaces:

- operator roster
- portal roster

Editing opens the matching modal with prefilled values. The page body keeps filters and identity KPIs, while workspace posture for portal users becomes contextual information inside the portal-user modal or a slim side surface.

### Tenants

Break the page into management zones:

- tenant registry
- project registry
- gateway key inventory

Primary actions move to toolbar buttons:

- `New tenant`
- `New project`
- `Issue gateway key`

Each action opens a dedicated modal. The selected project posture summary remains visible as a compact info surface rather than sharing space with forms.

### Coupons

Coupons become a campaign workbench:

- toolbar with search and status filters
- primary CTA `Create coupon`
- roster table with edit/archive/delete
- compact campaign guidance surface

Coupon creation and editing use the same modal so the roster remains the page anchor.

## Architecture

This refactor should stay inside `apps/sdkwork-router-admin` and avoid touching the admin shell contract.

Main changes:

- expand `sdkwork-router-admin-commons` with dialog and form primitives
- refactor feature packages to modal-driven mutation flows
- update tests so inline CRUD forms are no longer considered valid backend UX

## Testing Strategy

Use source-level tests first to lock in product rules:

- `Users` page contains dialog/modal flows for operator and portal creation
- `Tenants` page exposes modal-driven tenant/project/key actions
- `Coupons` page exposes modal-driven campaign editing
- page sources no longer contain large inline submit forms as the primary create path

Then verify:

- `pnpm --dir apps/sdkwork-router-admin typecheck`
- `pnpm --dir apps/sdkwork-router-admin build`
- `node --test tests/*.mjs` in `apps/sdkwork-router-admin`

## Completion Standard

This work is complete only when:

- no CRUD-heavy admin list page relies on always-visible create forms
- create/edit flows use modals or dedicated focused surfaces
- tables remain the primary registry experience
- visual and interaction quality clearly reads as a backend workbench
- tests, typecheck, and build all pass
