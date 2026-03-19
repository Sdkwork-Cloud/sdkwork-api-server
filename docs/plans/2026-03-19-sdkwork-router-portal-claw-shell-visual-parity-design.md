# SDKWork Router Portal Claw Shell Visual Parity Design

**Date:** 2026-03-19

## Goal

Make `apps/sdkwork-router-portal` match the visual shell, theme behavior, and settings experience of `claw-studio` as closely as possible while preserving Portal-specific routes, data, and business copy.

The target is not a loose inspiration pass. The target is shell parity:

- header rhythm and chrome feel like `claw-studio`
- sidebar density, states, collapse behavior, and resize behavior feel like `claw-studio`
- config center feels like a first-party sibling of `claw-studio Settings`
- theme mode and theme color behavior stay synchronized across shell and content
- the right side remains the Portal content canvas

## Chosen Approach

Use **visual-shell parity with Portal-owned business content**.

That means:

- reuse the `claw-studio` shell visual grammar
- keep Portal route labels, workspace data, and product actions
- avoid transplanting unrelated `claw-studio` product features such as command palette or instance management

This is the best trade-off because it delivers the requested visual sameness without forcing Portal into the wrong product semantics.

## Product Decisions

### Header

The Portal header should adopt the same high-level structure as the `claw-studio` titlebar:

- glass titlebar surface
- brand block on the left
- centered workspace context slot
- window controls on the right in Tauri mode

Portal does not need to copy `claw-studio` search or instance management behavior. Instead, the centered slot should show the active Portal workspace context using the same visual density and spacing language.

### Sidebar

The Portal sidebar should align with the `claw-studio` rail:

- same collapsed width target
- same expanded width envelope
- same section-label rhythm
- same active row treatment
- same floating collapse affordance
- same drag-resize affordance

To reduce drift from `claw-studio`, the dedicated top workspace summary block should be removed from the rail. Workspace identity should live in the header center slot and in the footer profile dock instead.

### Profile Dock

The footer profile dock remains the Portal entry for:

- settings
- sign out
- workspace identity

It should keep its Portal-specific actions, but visually stay in the same family as the `claw-studio` footer account control.

### Config Center

The config center should be rebuilt as a `claw-studio Settings`-style workspace:

- left rail for search and section navigation
- right pane for section content
- sections styled like `claw-studio` settings cards

Sections:

- `Appearance`
- `Navigation`
- `Workspace`

Portal-specific additions such as a live shell preview are allowed as long as the page still reads like a sibling of `claw-studio Settings`.

### Theme Contract

Theme state should stay equivalent to `claw-studio`:

- `themeMode`: `light | dark | system`
- `themeColor`: `lobster | tech-blue | green-tech | zinc | violet | rose`
- default mode: `system`
- default color: `lobster`
- root state applied through `data-theme`
- dark mode applied through `.dark`

The shell should also inherit the same scrollbar behavior and color-scheme treatment so native and browser-controlled surfaces feel consistent.

## Architecture

### Files Expected To Change

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/AppHeader.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/WindowControls.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/lib/portalPreferences.ts`
- `apps/sdkwork-router-portal/src/theme.css`
- Portal shell parity tests in `apps/sdkwork-router-portal/tests`

### What Must Stay Stable

- Portal auth and workspace APIs
- Portal route map
- main right-side content area ownership
- current persisted theme mode and theme color behavior
- sidebar collapse/expand capability

## Testing Strategy

This should be implemented with test-first string-contract updates because the shell parity requirements are mostly structural and style-contract based.

New or updated tests should verify:

- header now uses the `claw-studio` glass titlebar structure
- header exposes centered workspace context instead of a custom contrast bar treatment
- sidebar no longer renders the old top workspace summary block
- sidebar widths align with `claw-studio` dimensions
- config center uses `claw-studio Settings`-style sections and navigation
- theme stylesheet includes `claw-studio`-style scrollbar and dark color-scheme handling

## Completion Standard

This iteration is complete only when:

- header, sidebar, config center, and theme behavior read as the same shell family as `claw-studio`
- Portal content still renders correctly on the right
- theme changes remain synchronized across shell and content
- sidebar collapse and resize still work
- fresh tests, typecheck, and build complete successfully
