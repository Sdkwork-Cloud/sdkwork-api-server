# SDKWork Router Portal Claw Shell Alignment Design

**Date:** 2026-03-18

## Goal

Rebuild `apps/sdkwork-router-portal` so its shell architecture, visual language, theme behavior, configuration center, and sidebar interactions align closely with the `claw-studio` standard, while preserving the current portal feature surface and product semantics.

## Context

The current portal has already grown into a real product surface, but its application shell still differs materially from `claw-studio`.

Current gaps:

- the app shell lives mostly inside a single `sdkwork-router-portal-core/src/index.tsx` file
- routing still uses hash-based state instead of a real application router
- the visual system is driven by `portalx-*` bespoke CSS rather than a reusable shell theme contract
- there is no unified theme manager or configuration center equivalent to `claw-studio`
- the sidebar is visually different and does not support the same collapse and shell interaction model

At the same time, the runtime host already supports `/portal/<route>` style SPA navigation, so the portal can safely move to a `react-router-dom` application shell under the `/portal/` base path.

## Chosen Approach

### Recommendation

Adopt a **shell-convergence** approach:

- keep the current portal business modules and route semantics
- refactor `sdkwork-router-portal-core` into a shell package shaped like `sdkwork-claw-shell`
- align theme tokens, layout composition, sidebar behavior, and settings behavior with `claw-studio`
- restyle the business pages so they live inside the new shell rather than preserving the old `portalx-*` marketing-workbench hybrid shell

### Alternatives Considered

#### 1. Visual-only reskin

This would replace CSS and colors while leaving the current hash router and monolithic shell intact.

Why not chosen:

- too shallow to satisfy the requirement for framework and page architecture parity
- would keep theme and config behavior inconsistent
- would make future parity drift inevitable

#### 2. Full direct transplant of `claw-studio` packages

This would copy or mirror `sdkwork-claw-shell`, `sdkwork-claw-core`, and `sdkwork-claw-settings` structure almost literally.

Why not chosen:

- too invasive for the current dirty workspace
- risks dragging in unrelated desktop, i18n, and community-workspace assumptions
- would slow delivery without improving the portal business experience

#### 3. Shell convergence with portal feature preservation

This is the chosen option because it gives us:

- near-parity in shell and theme architecture
- lower risk to portal feature behavior
- a maintainable path for future convergence with the `claw-studio` standard

## Target Architecture

`sdkwork-router-portal-core` becomes the portal shell package in everything but name.

Target internal structure:

```text
packages/sdkwork-router-portal-core/src/
  index.tsx
  application/
    app/
      PortalProductApp.tsx
    layouts/
      MainLayout.tsx
    providers/
      AppProviders.tsx
      ThemeManager.tsx
    router/
      AppRoutes.tsx
      routePaths.ts
      routeManifest.ts
  components/
    AppHeader.tsx
    Sidebar.tsx
    ConfigCenter.tsx
    ShellStatus.tsx
  store/
    usePortalShellStore.ts
  lib/
    portalPreferences.ts
```

Key decisions:

- migrate from hash routing to `react-router-dom` with `BrowserRouter` and `basename="/portal"`
- centralize shell state in a single portal shell store
- keep business package boundaries intact and render their pages inside the new shell layout
- preserve public portal route keys and navigation labels, but route them through router path definitions instead of hash strings

## Theme System

The portal theme system will follow the `claw-studio` pattern:

- `ThemeManager` writes `data-theme` and `dark` state onto `document.documentElement`
- theme palettes are expressed as CSS custom properties and mapped into Tailwind theme tokens
- the shell defaults to the same dark workbench posture as `claw-studio`
- multiple theme accents are supported through named presets rather than one-off page colors

Planned theme contract:

- `themeMode`: `light | dark | system`
- `themeColor`: at minimum `tech-blue`, `lobster`, `green-tech`, `zinc`, `violet`, `rose`
- `sidebarCollapsed`
- `sidebarWidth`
- `hiddenSidebarItems`

Persistence strategy:

- store under a dedicated local key, for example `sdkwork-router-portal.preferences.v1`
- hydrate shell preferences before first interactive render
- keep the implementation local and synchronous so shell state is available immediately

## Configuration Center

The portal needs a configuration center with behavior highly consistent with `claw-studio`.

The first implementation will include:

- theme mode switching
- theme color switching
- sidebar navigation visibility toggles
- sidebar interaction preferences
- shell-level account and workspace summary placement consistent with the new layout

The configuration center does not replace current business pages such as billing or account. Instead, it becomes the shell-level control surface for presentation and workspace navigation preferences, while business pages keep domain-specific controls.

## Sidebar and Layout Behavior

The sidebar should feel functionally equivalent to `claw-studio` while remaining branded for the portal.

Required behavior:

- left-anchored dark shell sidebar
- click-to-collapse and click-to-expand
- desktop drag-resize support
- sticky shell behavior on larger screens
- mobile fallback that collapses into a compact drawer behavior
- active-route indicator and section grouping
- right side is always the main content surface

Layout composition:

- top app header
- left sidebar
- right main content region
- shell header within the content region for route title, workspace status, and top-level quick actions

## Business Page Refactor Scope

The business pages will stay functionally the same, but their composition and shell integration will change.

This work includes:

- rewrapping dashboard, routing, usage, account, billing, credits, user, and api-key pages in the new shell components
- aligning hero sections, metrics, tab lists, cards, tables, charts, and empty states to the shell theme contract
- removing dependence on the old `portalx-*` outer shell layout

This work does not include:

- changing backend portal APIs
- changing the domain intent of existing business routes
- importing unrelated `claw-studio` feature modules

## Testing Strategy

The refactor will follow TDD for shell behavior and parity-sensitive decisions.

Planned verification layers:

- architecture tests for the new shell package structure
- tests for router migration and route manifest integrity
- tests for theme manager and configuration center presence
- tests for sidebar collapse and shell navigation behavior
- existing portal polish tests updated to assert the new shell wording and structure
- `pnpm --dir apps/sdkwork-router-portal typecheck`
- `pnpm --dir apps/sdkwork-router-portal build`
- targeted `node --test` runs for the portal test suite

## Risks and Mitigations

### Risk: deep-link routing regression

Mitigation:

- use the existing runtime-host SPA fallback behavior already covered by `web_runtime_routing.rs`
- verify local build output still resolves for `/portal/<route>`

### Risk: dirty workspace overlap

Mitigation:

- work compatibly inside the current workspace
- avoid reverting existing unrelated changes
- scope edits to the portal app and only touch shared files when necessary

### Risk: theme parity without full shell transplant

Mitigation:

- copy the theme contract and shell interaction model, not just colors
- structure the portal core package to mirror the `claw-studio` shell shape

## Completion Standard

This work is only complete when all of the following are true:

- the portal uses a shell architecture clearly converged with `claw-studio`
- the sidebar visually and behaviorally matches the reference direction
- the configuration center supports consistent theme switching and sidebar configuration
- the right-side content area behaves as the primary work surface
- portal feature routes still function
- fresh tests, typecheck, and build evidence confirm the refactor
