# SDKWork Router Admin Shell And Settings Parity Design

## Goal

Refactor `apps/sdkwork-router-admin` so its shell architecture, theme pipeline, settings-center information architecture, and sidebar behavior align with `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\claw-studio` as a sibling product rather than a one-off visual imitation.

## Confirmed Direction

The user explicitly approved a full shell-plus-settings reconstruction, not a CSS-only facelift.

That means we are aligning all of the following together:

- shell package structure
- root bootstrap flow
- theme manager integration
- settings-center composition
- sidebar collapse and resize behavior
- right-side content-canvas contract

## Why The Current State Is Not Enough

`router-admin` already has:

- a claw-like dark sidebar
- route-based navigation
- theme mode and theme color controls
- a shell-level settings page

But it still diverges from `claw-studio` in the places that matter for long-term consistency:

- `sdkwork-router-admin-shell` is still a flat package instead of `application / components / styles`
- the root app imports `src/theme.css` directly instead of letting the shell own styles
- the root bootstrap does not mirror the shell bootstrap flow used by `claw-studio`
- `sdkwork-router-admin-settings` is still a single-file implementation instead of a real settings-center package
- shell preferences and settings information architecture are visually similar but not structurally converged

## Approaches Considered

### 1. Keep the current shell and only polish visuals

Pros:

- least code churn
- fastest short-term polish

Cons:

- does not satisfy framework-level consistency
- settings center remains a custom one-off
- future parity drift is guaranteed

### 2. Converge the admin shell and settings packages to the claw structure

Pros:

- matches the integration model the user asked for
- keeps admin business modules intact
- makes theme, sidebar, and settings behavior auditable and reusable
- reduces future divergence

Cons:

- requires package restructuring and test updates

### 3. Hard-port the claw implementation wholesale

Pros:

- maximum code reuse

Cons:

- drags unrelated product concepts into admin
- introduces unnecessary dependencies and maintenance burden

## Recommendation

Use approach 2.

We should align architecture and product behavior with `claw-studio`, but keep admin-specific routes, workbench state, and business pages local to `router-admin`.

## Target Architecture

### Root App

Target root behavior:

- `src/App.tsx` only renders `AppRoot` from `sdkwork-router-admin-shell`
- `src/main.tsx` mirrors `claw-web` by calling a shell bootstrap function before mount
- shell-owned styles move from root `src/theme.css` into `sdkwork-router-admin-shell/src/styles/index.css`

### Shell Package

Target structure:

```text
packages/sdkwork-router-admin-shell/src/
  application/
    app/
      AppRoot.tsx
    bootstrap/
      bootstrapShellRuntime.ts
    layouts/
      MainLayout.tsx
    providers/
      AppProviders.tsx
      ThemeManager.tsx
    router/
      AppRoutes.tsx
      routePaths.ts
  components/
    AppHeader.tsx
    Sidebar.tsx
    ShellStatus.tsx
  styles/
    index.css
  desktopWindow.ts
  index.ts
```

Responsibilities:

- `application/*` owns bootstrap, router, providers, and layout
- `components/*` owns shell-level UI
- `styles/index.css` owns the visual contract

### Settings Package

Target structure:

```text
packages/sdkwork-router-admin-settings/src/
  Settings.tsx
  GeneralSettings.tsx
  AppearanceSettings.tsx
  NavigationSettings.tsx
  WorkspaceSettings.tsx
  Shared.tsx
  index.ts
```

The settings center should use the same split layout pattern as `claw-studio`:

- left navigation rail
- search box
- query-param driven active tab
- right detail panel

The content remains admin-shell specific, but the information architecture becomes claw-consistent.

## Product Design

### Settings Navigation

Recommended tabs:

- `general`
  - shell overview
  - persistence summary
  - operator and workspace framing
- `appearance`
  - theme mode
  - theme color
  - live shell preview
- `navigation`
  - visible sidebar items
  - collapse posture preview
  - right-canvas explanation
- `workspace`
  - shell posture KPIs
  - content-region rules
  - control-plane continuity notes

### Sidebar Contract

The sidebar must stay behaviorally consistent with `claw-studio`:

- dark premium rail on the left
- click collapse and expand
- drag resize on desktop
- grouped sections
- active route indicator
- persistent hidden-item preferences
- profile and settings affordance anchored at the bottom

### Main Layout Contract

The right side is always the primary content display surface.

Layout rules:

- header above
- sidebar left
- content canvas right
- settings page also renders inside that same content canvas

## Theme Contract

The shell should keep the same theme ids as `claw-studio`:

- `tech-blue`
- `lobster`
- `green-tech`
- `zinc`
- `violet`
- `rose`

And the same theme modes:

- `light`
- `dark`
- `system`

The theme manager should:

- write `data-theme`
- write dark/light state classes
- write sidebar collapsed state
- react to system color-scheme changes

## Completion Standard

This refactor is complete only when:

- shell package structure mirrors the claw package shape
- settings package is split into real sections, not a single file
- root bootstrap mirrors the claw shell bootstrap pattern
- shell styles are owned by the shell package
- sidebar behavior remains fully functional
- theme and settings preferences remain persistent
- all admin pages still render in the right-side canvas
- tests, typecheck, and build all pass
