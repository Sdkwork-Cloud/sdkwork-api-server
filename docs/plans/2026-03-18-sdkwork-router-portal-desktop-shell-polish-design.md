# SDKWork Router Portal Desktop Shell Polish Design

**Date:** 2026-03-18

## Goal

Refine `apps/sdkwork-router-portal` so its shell, theme system, app header, and sidebar product behavior align more tightly with the `claw-studio` desktop standard, with a cleaner desktop titlebar, a single shell settings entry, and a stronger sidebar footer experience.

## Problems To Solve

- the app header currently exposes repeated shell entry points through multiple config buttons and a centered workspace control
- the header behaves like a web dashboard toolbar instead of a desktop titlebar
- the sidebar footer is visually fragmented because profile, settings, and logout are rendered as separate bottom controls
- shell theme tokens exist, but the shell hierarchy still allows older presentation patterns to bleed into desktop chrome decisions

## Approved Product Direction

### Header

- the left side contains only the portal logo mark and product name
- the center stays visually quiet and drag-friendly
- the right side contains only `WindowControls` when running in Tauri desktop mode
- shell settings, workspace state, and theme labels are removed from the header

### Settings Entry

- the header contains no settings entry
- shell settings move behind the sidebar user dock only
- the config center remains the single shell-level configuration surface

### Sidebar

- the upper sidebar remains the primary navigation surface
- the lower sidebar becomes a single profile dock instead of separate settings and logout buttons
- expanded mode shows avatar, display name, and workspace or tenant context
- collapsed mode shows a single high-quality avatar trigger
- clicking the dock opens a compact profile action panel with shell settings and sign out

### Theme System

- the shell continues to use the shared `--portal-*` token contract
- header, sidebar, profile dock, config center, and content region consume the same semantic token layers
- shell accent, borders, elevated surfaces, contrast surfaces, and text roles should be defined once and reused everywhere
- no shell chrome element should hardcode a local color when a semantic token already exists

## Target Architecture Changes

Primary files:

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/AppHeader.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx`
- `apps/sdkwork-router-portal/src/theme.css`
- `apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs`
- `apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs`

New shell component:

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx`

Component responsibilities:

- `AppHeader`: desktop titlebar only, brand left and window controls right
- `Sidebar`: navigation groups, collapse and resize controls, and footer docking slot
- `SidebarProfileDock`: user identity surface plus action menu for settings and sign out
- `ConfigCenter`: shell appearance and navigation preferences only, opened from the profile dock
- `MainLayout`: composes the new shell hierarchy and keeps the right side as the main content work area

## Interaction Model

### Desktop Header

- the main drag region should remain on the header shell
- product identity stays left-aligned to match desktop app expectations
- `WindowControls` remain isolated on the far right and are not visually mixed with product actions

### Sidebar Footer

- the profile dock should feel anchored and intentional, not like leftover utility buttons
- the action panel should inherit shell contrast styling so it reads as part of the desktop chrome
- settings and sign out become secondary actions nested under identity, which removes shell duplication

### Config Center

- the config center remains modal because it already houses appearance and navigation preferences
- its copy should stay aligned with the shell system instead of suggesting multiple shell entry points

## Theme Refactor Rules

- keep `--portal-shell-background`, `--portal-content-background`, `--portal-surface-background`, `--portal-surface-elevated`, `--portal-surface-contrast`, `--portal-sidebar-background`, `--portal-border-color`, `--portal-text-*`, and accent tokens as the canonical shell API
- add or refine tokens only when a shell surface cannot be expressed clearly with the existing set
- prefer semantic shell names over component-specific color names
- both light and dark desktop modes should preserve the same hierarchy:
  shell background -> chrome surface -> elevated cards -> accent state

## Testing Strategy

The next implementation pass should be protected by TDD:

- add failing shell parity tests for the simplified header contract
- add failing tests for removal of repeated header config entry points
- add failing tests for the single-entry sidebar profile dock behavior
- keep existing theme regression tests and extend them to cover any new shell token usage
- run `node --test apps/sdkwork-router-portal/tests/*.mjs`
- run `pnpm --dir apps/sdkwork-router-portal typecheck`
- run `pnpm --dir apps/sdkwork-router-portal build`

## Completion Standard

This polish pass is complete when:

- the header reads like a desktop titlebar, not a web toolbar
- settings no longer appear in the header
- the sidebar footer feels intentional and premium in both expanded and collapsed states
- the config center has a single clear entry path
- shell chrome styling remains fully token-driven
- fresh tests, typecheck, build, and runtime checks confirm the new behavior
