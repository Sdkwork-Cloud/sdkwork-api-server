# SDKWork Router Portal Claw Auth Shell Design

**Date:** 2026-03-19

## Goal

Make `apps/sdkwork-router-portal` reuse the `claw-studio` authentication and shell patterns as completely as possible while preserving the portal's real backend login, workspace, and dashboard APIs.

The target is product and framework parity, not a loose visual imitation:

- auth routes behave like `claw-studio`
- auth state is shared and persisted like `claw-studio`
- auth redirects and shell guards behave like `claw-studio`
- the sidebar, header, config center, desktop chrome, and content pane keep the current claw-style shell direction
- portal business pages continue to use real portal data

## Decision

Choose a **hybrid parity** approach:

- keep the portal backend authentication contract and token model
- replace the portal auth page flow and auth state ownership with a claw-style shared auth store and router-driven auth UX
- keep the existing claw-aligned shell token system, sidebar behavior, config center, and Tauri titlebar direction
- deepen parity where the current portal still diverges, especially auth routing, redirect restore, and auth-driven shell behavior

## Alternatives Considered

### Option A: Full transplant of `claw-studio` auth implementation

Pros:

- strongest implementation reuse
- lowest divergence in auth page internals

Cons:

- breaks the portal's real `/api/portal/auth/*` backend contract
- replaces a working backend login with the `claw-studio` local mock model
- fails the requirement for a real portal login system

### Option B: Keep current portal auth implementation and only reskin the pages

Pros:

- smallest diff
- lowest immediate risk

Cons:

- auth state remains owned by page-local/runtime-local code
- redirect restore still differs from `claw-studio`
- future shell drift remains likely
- does not satisfy "complete reuse" at the architectural level

### Option C: Rebuild portal auth around claw-style shared state and route semantics while keeping portal APIs

Pros:

- preserves real login and workspace fetching
- converges auth lifecycle, routing, and shell behavior with `claw-studio`
- makes shell/auth parity sustainable instead of cosmetic

Cons:

- touches portal core, auth package, route contracts, and tests together

This is the chosen option.

## Product Design

### Auth Routes

The portal auth surface should expose the same route semantics as `claw-studio`:

- `/login`
- `/register`
- `/forgot-password`
- `/auth` as a compatibility redirect to `/login`

The auth page derives its mode from the route rather than internal page-local mode state.

### Redirect Restore

Unauthenticated navigation to protected routes should redirect to:

- `/login?redirect=<requested-route>`

After successful login or registration, the app should navigate back to that requested route. Invalid or auth-only targets should resolve to `/dashboard`.

### Shared Auth State

Portal auth state should move into a dedicated persisted store in `sdkwork-router-portal-core`, shaped like the `claw-studio` auth store but backed by portal APIs.

The store should own:

- `isAuthenticated`
- `isBootstrapping`
- `user`
- `workspace`
- `sessionToken`
- `signIn`
- `register`
- `signOut`
- `hydrate`
- `syncWorkspace`
- `syncDashboard`

The shell runtime then consumes store state instead of manually duplicating auth lifecycle logic inside `PortalProductApp`.

### Auth Page Behavior

The portal auth page should follow the route-driven `claw-studio` behavior but use portal copy and portal API actions.

Required behavior:

- login mode submits to `loginPortalUser`
- register mode submits to `registerPortalUser`
- forgot-password mode is present as a first-class route and stays visually aligned with claw auth
- the page surface keeps the current portal product framing only where it does not conflict with claw shell grammar
- the user can switch between auth modes through route navigation instead of local toggle callbacks

Because there is no portal reset-password endpoint in the current backend contract, forgot-password stays a presentational route that guides the user back to login or registration. This is an intentional compatibility decision, not a missing implementation bug.

### Sidebar Footer And User Dock

The current profile dock direction is already aligned with `claw-studio` and should stay.

The next step is to make it auth-driven:

- authenticated user shows portal user identity and workspace context
- unauthenticated user navigates to `/login`
- sign-out always clears shared auth state and token, then routes to `/login`

### Header And Desktop Shell

The current desktop titlebar direction is already close to the target and should remain:

- brand on the left
- drag region in the center shell
- `WindowControls` on the right in Tauri mode

No additional auth actions should be added back into the header.

## Architecture

### Target File Ownership

Auth state and routing changes should be centered in:

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/app/PortalProductApp.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/routePaths.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/store/usePortalAuthStore.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-auth/src/pages/AuthPage.tsx`

Existing shell parity files remain part of the same product contract:

- `.../ThemeManager.tsx`
- `.../MainLayout.tsx`
- `.../Sidebar.tsx`
- `.../SidebarProfileDock.tsx`
- `.../AppHeader.tsx`
- `.../ConfigCenter.tsx`

### Backend Boundary

All real authentication and workspace data continues to flow through:

- `sdkwork-router-portal-portal-api`

No auth logic should call raw `fetch` directly from the auth page or shell runtime once the new store is introduced.

### Route Contracts

`PortalAnonymousRouteKey` should expand to include:

- `login`
- `register`
- `forgot-password`

Route helpers and manifest logic should treat those as first-class anonymous routes.

## Testing Strategy

This iteration should be driven by failing tests first.

### New behavioral contracts

Add tests that require:

- a dedicated `usePortalAuthStore`
- `/auth` redirecting to `/login`
- `/forgot-password` existing as a route
- protected routes redirecting with a `redirect` query param
- the auth page resolving mode from `location.pathname`
- sign-out clearing auth state and routing to `/login`
- the sidebar profile dock using shared auth state instead of shell-local logout-only callbacks

### Verification stack

Run at minimum:

- focused `node --test` auth and shell parity suites
- `pnpm --dir apps/sdkwork-router-portal typecheck`
- `pnpm --dir apps/sdkwork-router-portal build`

If preview rendering is needed for confidence, capture fresh screenshots after the code-level verification passes.

## Risks And Mitigations

### Risk: auth parity becomes visual-only again

Mitigation:

- move auth lifecycle into a shared store first
- update routes and guards before page polish

### Risk: forgot-password route implies unsupported backend behavior

Mitigation:

- implement it as an explicit presentational mode
- keep copy honest that recovery currently routes back through login support flow

### Risk: portal bootstrapping regresses while moving auth state

Mitigation:

- keep workspace and dashboard hydration in the auth store
- preserve the existing session-expired event handling from `sdkwork-router-portal-portal-api`

## Completion Standard

This work is complete only when:

- portal auth uses claw-style route semantics
- portal auth state is shared and persisted centrally
- protected-route redirect restore works
- the sidebar user dock reflects shared auth state
- the header and desktop shell remain claw-aligned
- shell and content still respond to theme changes together
- fresh tests, typecheck, and build all pass
