# SDKWork Router Portal Claw Desktop Theme Design

**Date:** 2026-03-18

## Goal

Align `apps/sdkwork-router-portal` with the visual language, theme behavior, sidebar model, and desktop shell conventions of `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\claw-studio`, while preserving the current portal route surface and business modules.

The target is not a loose reskin. The target is shell parity:

- the portal defaults behave like `claw-studio`
- the left sidebar behaves like `claw-studio`
- the header behaves like a desktop title bar when running in Tauri
- the right pane remains the main content surface
- content components follow theme changes through shared tokens instead of page-local colors

## Product Requirements Confirmed

The approved product direction for this iteration is:

- fully reference the `claw-studio` visual effect and theme grammar
- keep sidebar display behavior consistent with the reference shell
- support click collapse and expand for the sidebar
- keep the framework integration direction consistent with the reference shell
- keep the right side as the content display region
- make content pages follow the active theme
- place the user avatar and settings icon at the bottom of the sidebar
- support Tauri desktop mode
- show a `WindowControls` component on the header right in desktop mode
- support minimize, maximize, and close
- keep refining and regression-checking until the effect is consistently aligned

## Chosen Approach

### Recommendation

Use a **shell-convergence** approach rather than a CSS-only reskin.

That means:

- keep `sdkwork-router-portal` business packages and routes
- keep `sdkwork-router-portal-core` as the shell boundary
- replace shell defaults, shell layout, shell components, and shell tokens so they follow `claw-studio`
- make content pages consume the shared shell token system
- add a real desktop host boundary for Tauri instead of only showing desktop-like controls in the browser

### Alternatives Considered

#### 1. CSS-only parity pass

Pros:

- fastest to ship
- smallest diff

Cons:

- shell structure still diverges
- desktop behavior still incomplete
- theme consistency remains fragile
- future drift is almost guaranteed

#### 2. Full package transplant from `claw-studio`

Pros:

- strongest parity

Cons:

- too invasive for the current workspace
- would drag in unrelated product assumptions
- would complicate portal ownership boundaries

#### 3. Shell convergence with portal-preserving modules

This is the chosen option because it provides:

- strong parity where the user cares most: theme, shell, desktop feel, sidebar, content surfaces
- a realistic implementation scope inside the current repository
- low risk to portal business logic and route semantics

## Architecture Direction

`sdkwork-router-portal-core` remains the shell package, but its shell behavior becomes explicitly claw-style.

Primary responsibilities:

- compose the app providers
- manage theme and shell preferences
- define the browser route manifest
- render the desktop header, sidebar, config center, shell status, and content frame
- isolate desktop integration behind a Tauri-aware boundary

The shell stays organized under:

```text
apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/
  application/
    app/
    layouts/
    providers/
    router/
  components/
  lib/
  store/
```

The current business packages stay in place and continue to own page-level product behavior.

## Theme System

The portal theme system will be normalized to the same behavioral contract used by `claw-studio`.

### Shared contract

- `themeMode`: `light | dark | system`
- `themeColor`: `lobster | tech-blue | green-tech | zinc | violet | rose`
- `isSidebarCollapsed`
- `sidebarWidth`
- `hiddenSidebarItems`

### Default behavior

Portal defaults will no longer use their own preference bias.

They will align to the reference shell:

- default `themeMode`: `system`
- default `themeColor`: `lobster`

### Token layering

`apps/sdkwork-router-portal/src/theme.css` will be reshaped into two layers:

1. `claw-studio`-style palette and shared shell primitives
2. portal-specific semantic tokens for shell, content, sidebar, dialog, and chart surfaces

Key token families:

- `--portal-shell-background`
- `--portal-content-background`
- `--portal-surface-background`
- `--portal-surface-elevated`
- `--portal-glass-background`
- `--portal-overlay-surface`
- `--portal-sidebar-background`
- `--portal-sidebar-text`
- `--portal-border-color`
- `--portal-text-primary`
- `--portal-text-secondary`
- `--portal-chart-grid`
- `--portal-chart-tooltip-background`

All page surfaces, cards, forms, tables, charts, dialogs, and pills must derive from these shared tokens.

## Config Center

The portal will keep a shell-level configuration center, but its visual system and behavior will converge with the reference shell.

It is responsible for:

- theme mode switching
- theme color switching
- sidebar visibility preferences
- shell preference persistence

It is not responsible for:

- billing domain settings
- account profile forms
- route-specific business preferences

Layout direction:

- appearance controls in the primary pane
- sidebar navigation controls in the secondary pane
- dialog shell, spacing, borders, elevation, and chip treatments consistent with the claw grammar

## Desktop Shell And Tauri Mode

The current portal has Tauri API helpers but no app-local Tauri host scaffold.

To satisfy the approved requirement, the portal must support both:

1. browser mode
2. real Tauri desktop mode

### Desktop host boundary

Add a new `src-tauri` directory under `apps/sdkwork-router-portal`, based on the local admin desktop setup as the repository reference.

Expected responsibilities:

- create a Tauri binary entry point
- define the window configuration
- allow the front-end to run under a desktop host
- expose the environment needed for title-bar drag regions and window commands

### Header behavior

The header becomes a real desktop shell header.

Rules:

- left side holds brand and workspace identity
- center area can show the active workspace surface
- right side shows `WindowControls` in desktop mode
- interactive controls are excluded from drag regions
- the desktop title bar feel must remain clean and not compete with page content

### `WindowControls`

A dedicated shell component will own:

- minimize
- maximize or toggle maximize
- close

The portal should not scatter these actions through the header implementation.

## Sidebar Model

The sidebar must match the reference shell in behavior, not just colors.

Required behavior:

- fixed left rail
- dark, high-contrast shell surface
- grouped navigation
- active item indicator
- click collapse and expand
- drag resize on desktop
- bottom identity area with avatar and settings icon
- right pane always remains the main content surface

Expanded state:

- workspace identity at the top
- grouped route navigation in the middle
- avatar block plus settings action at the bottom

Collapsed state:

- compact icon rail
- still exposes bottom avatar and settings actions
- still supports expand affordance

## Right Content Pane

The right side is the primary work surface.

That means:

- shell chrome must not visually overpower the page
- shell status surfaces should introduce the page, not replace it
- content containers should feel like part of one product family
- width, spacing, and background layering should stay consistent across dashboard, routing, usage, credits, billing, user, and account

## Content Theme Following

This iteration must enforce full theme following across content components.

Affected surface types include:

- cards
- tabs
- tables
- forms
- pills
- empty states
- charts
- dialogs
- buttons

The shared component layer in `sdkwork-router-portal-commons` becomes the enforcement point for theme consistency.

Business pages should not rely on page-local hard-coded shell colors for primary backgrounds, borders, or text hierarchy.

## Testing And Regression Strategy

The implementation is complete only after repeated regression checks.

### Regression pass 1: shell parity

Verify:

- header hierarchy
- desktop title bar feel
- sidebar collapse and expand
- sidebar resize handle
- sidebar footer avatar and settings icon
- config center shell parity

### Regression pass 2: theme switching

Verify:

- `light`
- `dark`
- `system`

Across:

- `lobster`
- `tech-blue`
- `green-tech`
- `zinc`
- `violet`
- `rose`

### Regression pass 3: page coverage

Verify the main portal pages:

- dashboard
- routing
- api keys
- usage
- user
- credits
- billing
- account

### Automated verification

Strengthen and run:

- `apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-navigation-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-form-ux-polish.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-dashboard-analytics.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-routing-polish.test.mjs`
- portal `typecheck`
- portal `build`

## Risks And Mitigations

### Risk: shell parity remains superficial

Mitigation:

- enforce shared token usage in commons
- add tests for desktop controls, default theme values, and sidebar footer semantics

### Risk: desktop behavior exists only in front-end code

Mitigation:

- add an actual `src-tauri` host scaffold under the portal app
- mirror the local admin app host shape where useful

### Risk: business pages drift from shell tokens

Mitigation:

- sweep shared primitives first
- then update page-level hard-coded surface colors

### Risk: dirty workspace collisions

Mitigation:

- keep changes scoped to `apps/sdkwork-router-portal` and the new plan docs
- avoid overwriting same-topic untracked docs already present in the workspace

## Completion Standard

This work is only complete when all of the following are true:

- the portal default theme behavior matches `claw-studio`
- the portal shell feels visually and behaviorally aligned with `claw-studio`
- the sidebar supports collapse, expand, resize, avatar footer, and settings access
- desktop mode shows a right-aligned `WindowControls` component and valid drag regions
- the right side remains the content display pane
- theme changes flow through shell and content components together
- Tauri host scaffolding exists for the portal app
- regression tests, typecheck, and build all confirm the aligned result
