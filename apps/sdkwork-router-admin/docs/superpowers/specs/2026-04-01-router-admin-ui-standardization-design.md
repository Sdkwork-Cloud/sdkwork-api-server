# Router Admin UI Standardization Design

**Date:** 2026-04-01

**Goal:** Rebuild `sdkwork-router-admin` on top of `@sdkwork/ui-pc-react` so the project uses one shared PC UI framework across the shell, every page, and every reusable component, while removing the local legacy UI system instead of carrying it forward.

## Summary

`sdkwork-router-admin` already ships a functioning admin product, but its presentation layer is split across a large local CSS system (`adminx-*`), a legacy local UI package (`sdkwork-router-admin-commons`), and page-specific composition patterns. That structure blocks convergence with the shared PC React design system and guarantees more UI drift over time.

The target state is a direct adoption of `@sdkwork/ui-pc-react` as the only standard UI source. The admin app will keep its current routing model, business data flow, and operator workflows, but the entire shell and page layer will be rebuilt around shared SDKWORK shell, workbench, form, overlay, feedback, and data-display primitives.

This is not a compatibility migration. The old local UI layer is removed, not wrapped for long-term coexistence.

## Current State

### Workspace shape

- The app lives at `apps/sdkwork-api-router/apps/sdkwork-router-admin`.
- It is an isolated pnpm workspace with `packages/*` as local packages.
- Business features are already decomposed into packages such as `overview`, `users`, `tenants`, `coupons`, `catalog`, `traffic`, `operations`, `settings`, and `apirouter`.
- The shell is owned by `sdkwork-router-admin-shell`.
- Shared local UI is currently owned by `sdkwork-router-admin-commons`.

### Presentation problems

- `sdkwork-router-admin-commons` is effectively a local design system.
- `packages/sdkwork-router-admin-shell/src/styles/index.css` contains a very large `adminx-*` visual language and shell contract.
- Feature pages depend on local wrappers such as `Surface`, `PageToolbar`, `Pill`, `InlineButton`, `FormField`, and local dialog composition.
- Existing tests encode many details of the local implementation instead of validating the future shared framework contract.

### Useful constraints

- The feature package split is already valuable and should remain.
- Current business behavior, routing, and data wiring should be preserved.
- Many pages already follow a table-first and dialog-first admin workflow, which maps well onto the shared workbench patterns in `@sdkwork/ui-pc-react`.

## Design Principles

- `@sdkwork/ui-pc-react` is the only approved source for generic UI primitives and patterns.
- The admin app may compose shared UI, but it must not grow a second local generic UI framework.
- Old visual systems are deleted, not adapted into a permanent bridge.
- Business packages keep their boundaries; the rebuild is focused on presentation architecture.
- Shared page patterns must be consistent across the entire admin product.
- Page JSX should describe business intent, not rebuild layout or style primitives.

## Non-Goals

- Rewriting admin-control-plane APIs or domain models.
- Replacing the route graph or auth behavior.
- Introducing a second local component library as an adapter facade.
- Preserving `adminx-*` classes, legacy primitives, or legacy structure tests for backward compatibility.

## Target Architecture

### 1. Shared UI framework

`@sdkwork/ui-pc-react` becomes the only generic UI dependency for:

- application shell patterns
- workspace and workbench patterns
- settings center patterns
- buttons, badges, dialogs, drawers, forms, toolbars, filters, tables, cards, and feedback
- theme tokens and shared stylesheet

The admin app imports the shared framework stylesheet once at the app root and stops maintaining a local global visual language beyond minimal host-level resets that are strictly app-specific.

### 2. Local app structure

The future local structure has three presentation layers:

- `core`
  - routes
  - auth and session state
  - application store
  - business-side selectors and view-model helpers
- `shell`
  - top-level providers
  - route composition
  - navigation metadata
  - application shell assembly
  - desktop-host integration
- `feature packages`
  - business-page containers
  - page-specific compositions
  - page dialogs and action handlers

What does not exist in the target state:

- a permanent replacement for `sdkwork-router-admin-commons`
- a new local generic primitive package
- large app-local global CSS for surfaces, forms, tables, shell chrome, and auth layouts

### 3. Allowed local composition

The app may still define admin-specific compositions, but only as business-facing combinations of shared framework parts. Examples:

- `AdminShellFrame`
- `AdminNavigationRail`
- `AdminPageFilters`
- `AdminMetricDeck`
- `AdminEntityDialogs`

Those local compositions must not redefine generic button, input, table, or modal behavior.

## Dependency and Build Strategy

### Direct framework adoption

Because this app is an isolated pnpm workspace, the shared framework should be added as a direct dependency from the sibling path instead of creating a local compatibility library.

The implementation should:

- add `@sdkwork/ui-pc-react` as a direct dependency from the sibling package path
- ensure Vite and TypeScript resolve the package cleanly in local development
- keep one React runtime and avoid duplicate React copies in the dependency tree
- import `@sdkwork/ui-pc-react/styles.css` once at the root entry

If source aliasing is required for local iteration speed, it should still preserve the package import contract:

- app code imports `@sdkwork/ui-pc-react`
- Vite and TypeScript may map that package name to the sibling source during development

That preserves the real dependency boundary without inventing a second local UI package.

## Shell and Provider Model

### Root providers

The root app should own:

- `SdkworkThemeProvider`
- router provider
- toaster provider
- existing admin i18n provider, if still needed
- any desktop host or window integration providers

The current theme bootstrapping should be rebuilt so shared SDKWORK theme tokens drive the shell and page visuals. The local theme manager may survive only as app state and persistence glue, not as a visual system owner.

### Shell target

The current authenticated shell should be rebuilt using shared shell patterns such as:

- `AppShell`
- `DesktopShellFrame`
- `WorkspaceScaffold`
- shared `PageHeader`
- shared navigation and toolbar primitives

The auth route split remains, but the authenticated app shell no longer depends on `adminx-shell` classes or legacy shell CSS.

## Page Pattern Standard

Every page category is rebuilt onto one shared pattern family.

### Overview

Target pattern:

- `PageHeader`
- `StatCard`
- `WorkspacePanel` or `Card`

The page becomes the standard reference for dashboard summary styling in this app.

### Users, Tenants, Coupons

Target pattern:

- `CrudWorkbench`

Each page uses one consistent structure:

- page header
- filter/search actions
- main data table
- create/edit dialogs
- destructive confirmation dialog

### Catalog and API Router pages

Target pattern:

- `ManagementWorkbench`
- `CrudWorkbench`
- `DetailDrawer` or `InspectorRail` when secondary context is needed

These pages are information-dense and should standardize on a main roster plus focused secondary detail/configuration surface.

### Traffic and Operations

Target pattern:

- `ManagementWorkbench`

These pages must share one consistent filter, table, status, empty-state, and dense-data layout language.

### Settings

Target pattern:

- `SettingsCenter`

Settings becomes a first-class shared settings experience instead of a local one-off preferences page.

### Auth

Target pattern:

- shared form, card, panel, and feedback primitives from `@sdkwork/ui-pc-react`

The current auth stage is rebuilt visually and structurally instead of preserving `adminx-auth-*`.

## Component Mapping Rules

Legacy local abstractions must be replaced with shared equivalents.

| Legacy local abstraction | Target shared abstraction |
| --- | --- |
| `Surface` | `WorkspacePanel` or `Card` |
| `PageToolbar`, `ToolbarInline`, `ToolbarField`, `ToolbarSearchField` | shared toolbar and filter primitives |
| `Pill` | `Badge` or `StatusBadge` |
| local `DataTable` contract | shared `DataTable` |
| `InlineButton` | shared `Button`, `ToolbarButton`, or `IconButton` |
| local dialog composition | shared overlays (`Dialog`, `Modal`, `ConfirmDialog`, `Drawer`) |
| local `FormField` | shared form primitives |
| local settings layout | `SettingsCenter` |
| local CRUD page structures | `CrudWorkbench` or `ManagementWorkbench` |

Pages may add business labels, copy, and column renderers, but the structural UI contract is shared.

## Data Flow and State Rules

- Existing business data sources and route behavior remain intact.
- View-model transformation stays local to the feature package or a feature-local helper.
- Global state remains limited to application concerns such as session, theme preference, shell posture, and navigation state.
- Page-local state stays local, including:
  - search text
  - active filters
  - dialog state
  - selected row
  - pending destructive actions
- Shared UI components remain business-agnostic and receive clean props.
- Visual semantics such as badge tone, panel variant, and table decorations are derived in the presentation layer, not encoded into domain records.

## Code Removal Plan

The following legacy presentation assets are removed as part of the migration:

- `sdkwork-router-admin-commons` as a long-lived shared UI package
- the `adminx-*` shell and auth styling system
- local page primitives that duplicate shared framework components
- tests that only assert presence of legacy local implementation details

The following may remain only if still tied to domain behavior instead of visual ownership:

- route definitions
- workbench state hooks
- i18n utilities
- desktop-host integration
- feature view-model helpers

## Migration Order

The implementation order should optimize architectural convergence rather than visible short-term progress.

1. **Dependency and app-root reset**
   - add `@sdkwork/ui-pc-react`
   - wire package resolution
   - import shared stylesheet
   - replace root provider and theme composition
2. **Shell rebuild**
   - rebuild app shell, header, navigation, and authenticated layout using shared shell patterns
   - remove legacy shell CSS ownership
3. **Settings and auth rebuild**
   - use `SettingsCenter` and shared auth form composition
   - these pages define the shared control, form, and settings standard for the rest of the app
4. **Overview rebuild**
   - establish the dashboard card and workspace panel standard
5. **Standard CRUD rebuild**
   - `users`
   - `tenants`
   - `coupons`
6. **Complex workbench rebuild**
   - `catalog`
   - `apirouter`
7. **Dense-data rebuild**
   - `traffic`
   - `operations`
8. **Legacy removal and final polish**
   - delete old UI modules and styles
   - rewrite tests to validate the shared framework architecture
   - run final consistency and UX polish passes

## Testing and Verification Strategy

The verification target is architectural correctness plus working product behavior.

### Required checks

- `pnpm typecheck`
- `pnpm build`
- existing runtime or behavior tests that remain valid after the rebuild
- new architecture tests that verify the app now depends on `@sdkwork/ui-pc-react` and no longer depends on the removed local UI system

### Test rewrite direction

Old tests that assert `adminx-*` class names, legacy local wrapper names, or old local shell details should be replaced with tests that assert:

- the shared framework is imported and used
- legacy local UI packages are gone from feature imports
- page structures are built on shared workbench/settings/shell patterns
- core workflows still render the required business actions and states

## Risks and Controls

### Risk: duplicate React or package resolution instability

Control:

- keep a single package import contract for `@sdkwork/ui-pc-react`
- verify dependency graph and bundler resolution early

### Risk: CSS collision during the transition

Control:

- import the shared framework stylesheet at the root first
- remove legacy global UI CSS aggressively instead of letting two systems coexist

### Risk: behavior regressions while replacing page structure

Control:

- preserve feature package boundaries
- keep data and action handlers stable
- rebuild presentation around existing business contracts

### Risk: incomplete migration leaves hybrid UI

Control:

- treat legacy UI removal as part of the definition of done
- do not mark the work complete while old shell/page systems still own major surfaces

## Definition of Done

The migration is complete only when all of the following are true:

- `sdkwork-router-admin` uses `@sdkwork/ui-pc-react` as its shared UI standard
- legacy local generic UI ownership has been removed
- the authenticated shell is rebuilt on shared shell/workspace patterns
- `overview`, `users`, `tenants`, `coupons`, `catalog`, `traffic`, `operations`, `settings`, and `apirouter` all use shared page patterns
- auth is rebuilt on shared form and panel primitives
- legacy `adminx-*` UI styling is removed from primary ownership
- typecheck and build succeed
- the updated tests validate the new architecture and critical product workflows

## Recommendation

Proceed with a full direct UI standardization using `@sdkwork/ui-pc-react` as the only generic UI framework, preserving business logic and package boundaries while rebuilding the shell and every page on top of shared shell, settings, workbench, table, form, overlay, and feedback patterns.
