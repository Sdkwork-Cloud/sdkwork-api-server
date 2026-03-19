# Router Admin Claw Auth Shell Design

## Goal

Make `apps/sdkwork-router-admin` feel and behave like a true sibling of `claw-studio` by aligning the admin login surface, shell composition, sidebar behavior, theme pipeline, and settings center with `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\claw-studio`, while preserving the admin app's real backend authentication and live control-plane workflows.

## Approved Direction

The user approved the following constraint:

- keep `sdkwork-router-admin` on its existing real admin authentication API
- fully align login visuals, auth route shape, shell layout, sidebar behavior, theme configuration, and settings-center experience with `claw-studio`
- make the right side the only content canvas
- iterate until the admin shell is materially consistent with the reference product

## What Is Already Reusable

The current admin app already mirrors much of the target package graph:

- `sdkwork-router-admin-core` already owns persisted theme and sidebar state
- `sdkwork-router-admin-shell` already owns providers, router, layout, header, sidebar, and shell styles
- `sdkwork-router-admin-settings` already provides a split settings center
- business modules already render inside a shared shell

This means the best path is not a full rewrite. The best path is a parity-focused reconstruction of three layers:

- auth surface
- shell surface
- theme and settings surface

## Product Design

### 1. Auth Surface

The admin app should adopt the same auth route shape as `claw-studio`:

- `/auth` redirects to `/login`
- `/login` renders the primary login mode
- `/register` renders the alternate account-request mode
- `/forgot-password` renders the password-recovery mode

The page composition should closely mirror `claw-studio`:

- two-column auth canvas
- branded QR / companion panel on the left
- large heading, concise helper copy, and icon-led fields on the right
- shared form framing and bottom mode-switch affordances

The behavioral difference is intentional and explicit:

- only `/login` submits to the real admin backend via `loginAdminUser`
- `/register` and `/forgot-password` keep the same visual flow but present admin-specific guidance rather than fake self-service mutations

This preserves truthfulness while keeping route shape and product rhythm aligned.

### 2. Shell Surface

The shell should remain route-driven and keep the admin business modules intact, but its composition should converge on the `claw-studio` shell contract:

- translucent header across the top
- persistent left sidebar with grouped navigation
- collapsible and resizable sidebar
- right-side content canvas as the only content region
- auth pages rendered outside the shell chrome but inside the same ambient page atmosphere

The sidebar must keep the stronger interaction language already started in admin:

- active route rail
- click collapse and expand
- resize affordance on desktop
- bottom-anchored account control and settings entry
- persistent collapsed state and width

### 3. Theme And Settings Surface

The theme contract must be identical to `claw-studio`:

- theme modes: `light`, `dark`, `system`
- theme colors: `tech-blue`, `lobster`, `green-tech`, `zinc`, `violet`, `rose`
- persistent store-backed preferences
- root-level `data-theme` and `dark` class handling

The admin shell keeps a richer shell-specific settings center than `claw-studio`, but its visual structure and preview language must still match the reference:

- left navigation rail with search
- right detail stage
- tactile cards and preview blocks
- sidebar preview, theme preview, and shell continuity language

## Architecture

### Routing

`sdkwork-router-admin-core/src/routePaths.ts` expands to include auth-route parity:

- `AUTH`
- `LOGIN`
- `REGISTER`
- `FORGOT_PASSWORD`

Protected shell routes stay unchanged for business modules.

### Auth Component Contract

`sdkwork-router-admin-auth` should expose an auth page that accepts:

- current auth mode
- current backend status
- submit/loading state
- real login callback

The auth package should own route-aware auth presentation so that shell routing remains thin.

### Shell Component Contract

`sdkwork-router-admin-shell` keeps ownership of:

- `MainLayout`
- `AppHeader`
- `Sidebar`
- `ThemeManager`
- `AppRoutes`
- `styles/index.css`

The shell package should continue exporting the same public surface so the root app stays minimal.

### Settings Contract

`sdkwork-router-admin-settings` remains split into focused sections:

- `GeneralSettings`
- `AppearanceSettings`
- `NavigationSettings`
- `WorkspaceSettings`

The changes are parity-oriented, not package-topology changes. The package already sits at the right abstraction level.

## Data Flow

### Login

1. Router resolves auth mode from pathname.
2. Auth page renders the claw-style auth surface.
3. When mode is `login`, submit calls `handleLogin`.
4. `handleLogin` continues to call `loginAdminUser`, persist the session token, and refresh live workspace data.
5. On success, the user is redirected to the overview route.

### Theme

1. Theme preferences live in `useAdminAppStore`.
2. `ThemeManager` mirrors store state onto the root element.
3. Shell and content primitives consume shared CSS variables and root classes.
4. Settings center mutates the store and therefore updates the live shell immediately.

### Sidebar

1. Sidebar collapse state, width, and hidden items live in `useAdminAppStore`.
2. Sidebar reads route metadata from `adminRoutes`.
3. Settings center mutates sidebar visibility and posture.
4. Main layout always renders the selected route into the right-side canvas.

## Error Handling

- Failed login must still show the live backend status message returned by `AdminWorkbenchProvider`.
- Unsupported register and forgot flows must never pretend success; they should redirect users toward the real admin onboarding path.
- Missing persisted theme values should continue to fall back to safe defaults through the existing store merge logic.

## Testing Strategy

Add or extend static contract tests so they prove:

- auth route parity exists for `/auth`, `/login`, `/register`, and `/forgot-password`
- auth page now includes claw-style two-panel composition and admin-specific non-fake handling for non-login modes
- shell continues to expose click-collapse and resize behavior
- settings center still exposes all six theme colors and shell preview affordances
- root theme manager still applies shared root classes and theme attributes

Verification after implementation should include:

- `node --test tests/*.mjs`
- `pnpm --dir apps/sdkwork-router-admin typecheck`
- `pnpm --dir apps/sdkwork-router-admin build`

## Completion Standard

This work is complete only when:

- login UI is recognizably aligned with `claw-studio`
- auth route shape matches `claw-studio`
- admin keeps using the real backend login path
- sidebar collapse, expand, and resize still work
- the right side remains the single content canvas
- settings center keeps live parity for theme and sidebar configuration
- admin tests, typecheck, and build all pass
