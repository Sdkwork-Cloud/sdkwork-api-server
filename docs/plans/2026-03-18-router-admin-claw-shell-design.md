# SDKWork Router Admin Claw-Shell Design

## Goal

Upgrade `apps/sdkwork-router-admin` from a standalone dark dashboard into a shell-based admin product that matches the `claw-studio` visual system, theming model, sidebar interaction model, and settings-center behavior while preserving all live admin workflows.

## Context

The current admin app already has good business-module separation:

- root `src/` only mounts the app
- business pages are isolated in `packages/`
- shared presentation primitives already flow through `sdkwork-router-admin-commons`

What is missing is the shell layer that `claw-studio` uses:

- theme state and theme manager
- route-driven shell instead of hash-driven page switching
- collapsible and resizable sidebar
- top application header
- settings center with theme and sidebar configuration
- unified surface tokens for light, dark, and accent themes

## User-Facing Target

The finished admin app should feel like a sibling product of `claw-studio`, not a separate prototype. The target experience is:

- left sidebar with section grouping, active markers, and click-to-collapse behavior
- right content area with the current page rendered inside a consistent shell
- top header with product branding and utility actions
- settings center that controls theme mode, theme color, and sidebar item visibility
- light, dark, and system theme behavior driven by persistent preferences
- the same primary color theme variants as `claw-studio`
- the same structural visual language: soft glass header, dark premium sidebar, elevated cards, restrained borders, and subtle ambient gradients

## Approaches Considered

### 1. CSS-only facelift on the current hash shell

Pros:

- smallest code churn
- keeps all current page logic intact

Cons:

- cannot reach `claw-studio` shell parity
- no real settings center integration
- no route-native sidebar state or deep-link behavior
- theme consistency would stay fragile

### 2. Full shell migration with new shell and settings packages

Pros:

- matches `claw-studio` package responsibilities and runtime flow
- gives us a persistent theme model and sidebar model
- allows future admin feature growth without rebuilding the app shell again
- keeps business pages mostly intact while upgrading presentation and routing

Cons:

- moderate refactor across app root, package graph, routes, and tests

### 3. Hard-port `claw-studio` implementation wholesale

Pros:

- maximum visual reuse

Cons:

- too much unrelated product baggage
- risks forcing chat/instance abstractions into the admin app where they do not belong
- weak fit for the admin domain

## Recommendation

Use approach 2.

That gives us shell-level parity with `claw-studio` while keeping the admin app domain-specific. We will mirror `claw-studio` architecture where it matters:

- `shell` package for layout/providers/router composition
- `core` package for persisted app UI state
- `settings` package for user preferences surfaces
- route-driven navigation with a consistent left-shell and right-content split

## Architecture

### Package Changes

Keep existing business packages and add two new packages:

- `sdkwork-router-admin-shell`
- `sdkwork-router-admin-settings`

Responsibilities:

- `sdkwork-router-admin-core`
  - app UI state store
  - theme types and sidebar preferences
  - route manifest
  - shell-facing helpers
- `sdkwork-router-admin-shell`
  - providers
  - theme manager
  - layout
  - header
  - sidebar
  - route rendering
- `sdkwork-router-admin-settings`
  - settings page and tabs
  - theme mode and color controls
  - sidebar item visibility controls

### Routing Model

Replace hash navigation with `react-router-dom` and a browser router using the existing `/admin/` base path.

Route shape:

- `/login`
- `/overview`
- `/users`
- `/tenants`
- `/coupons`
- `/catalog`
- `/traffic`
- `/operations`
- `/settings`

Routing rules:

- unauthenticated users are redirected to `/login`
- authenticated users land in the shell layout
- sidebar links and settings deep links use route paths instead of local hash state

### Theme Model

Mirror `claw-studio` theme behavior:

- theme mode: `light | dark | system`
- theme color:
  - `tech-blue`
  - `lobster`
  - `green-tech`
  - `zinc`
  - `violet`
  - `rose`
- preferences persisted in local storage

The theme manager will:

- set `data-theme`
- set a mode attribute/class on the root element
- react to system dark-mode changes when mode is `system`

### Sidebar Model

Sidebar behavior should match `claw-studio` expectations:

- collapsible by explicit click
- width persisted
- optional resize handle
- section-grouped navigation
- active route highlight
- hidden items controlled by settings center

The sidebar keeps the dark premium treatment even when the content area is light, which is one of the strongest recognizers of the `claw-studio` visual style.

### Visual System

The admin app should reuse the same design language rather than its current industrial dark dashboard language.

Core visual rules:

- ambient page background with layered radial gradients
- translucent header with blur
- dark, vertically segmented sidebar
- neutral content canvas with elevated cards
- primary accents only from the selected theme
- restrained border contrast
- rounded corners aligned with `claw-studio`

Existing admin primitives will be restyled rather than replaced:

- `SectionHero`
- `Surface`
- `StatCard`
- `Pill`
- `DataTable`
- `InlineButton`

This keeps business pages intact while bringing their appearance into parity.

### Settings Center

The new settings page should follow the same split-panel structure as `claw-studio`:

- left settings navigation
- right scrolling detail panel

Initial tabs:

- `General`
  - theme mode
  - theme color
  - sidebar item visibility
- `Workspace`
  - admin-shell behavior summary
  - live workspace and operator hints

We deliberately keep the scope tight so the settings center is polished and complete rather than broad but shallow.

## Regression Standards

The migration is only acceptable when all of the following are true:

- shell uses route-based navigation
- sidebar supports collapse and expand via click
- right side is the sole content display region
- settings center can switch theme mode and color
- theme changes propagate to every page surface
- sidebar hidden-item preferences persist
- overview, users, tenants, coupons, catalog, traffic, operations, and settings all render inside the same shell
- login remains outside the shell
- architecture tests prove new packages and shell contracts exist

## Risk Management

Primary risks:

- breaking live admin workflows during shell refactor
- inconsistent theming across old page primitives
- accidental clashes with unrelated uncommitted repo changes

Mitigations:

- keep business-page logic largely unchanged
- migrate presentation through shared primitives and shell CSS
- add contract tests before production edits
- limit edits to `apps/sdkwork-router-admin`
