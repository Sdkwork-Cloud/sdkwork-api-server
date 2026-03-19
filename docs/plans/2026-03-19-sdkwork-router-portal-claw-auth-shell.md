# SDKWork Router Portal Claw Auth Shell Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rebuild the portal auth lifecycle so `apps/sdkwork-router-portal` uses `claw-studio`-style auth routing, shared auth state, redirect restore, and shell integration while preserving the portal's real backend login APIs and current claw-style shell parity.

**Architecture:** Introduce a shared `usePortalAuthStore` in portal core, move auth bootstrapping and sign-out into that store, route auth via `/login`, `/register`, and `/forgot-password`, and update the auth package to derive mode from the router. Keep the existing theme, sidebar, profile dock, config center, and desktop header token system, but wire them to the shared auth store for complete shell/auth convergence.

**Tech Stack:** React 19, TypeScript, React Router 7, Zustand, Tailwind CSS v4, Tauri 2, Node test runner

---

### Task 1: Lock Auth Route And Store Parity In Tests

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs`
- Create: `apps/sdkwork-router-portal/tests/portal-auth-parity.test.mjs`

**Step 1: Write the failing test**

Add assertions that require:

- `usePortalAuthStore.ts` to exist
- `/forgot-password` to exist in route keys and route paths
- `/auth` to redirect to `/login`
- protected routes to use a `redirect` query param
- auth page mode to come from the router rather than `onNavigate('login'|'register')` callbacks
- sidebar/profile dock to depend on shared auth state

**Step 2: Run test to verify it fails**

Run:

```powershell
node --test tests/portal-shell-parity.test.mjs tests/portal-theme-config.test.mjs tests/portal-auth-parity.test.mjs
```

Expected: FAIL because the shared auth store and claw-style auth route surface do not exist yet.

**Step 3: Write minimal implementation plan notes**

Capture the exact missing files and strings from the failure output before implementing.

**Step 4: Commit**

```bash
git commit apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs apps/sdkwork-router-portal/tests/portal-auth-parity.test.mjs -m "test: define portal auth shell parity"
```

### Task 2: Introduce The Shared Portal Auth Store

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/store/usePortalAuthStore.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/app/PortalProductApp.tsx`

**Step 1: Write the failing test**

Add or extend tests to require:

- `isAuthenticated`
- `isBootstrapping`
- `signIn`
- `register`
- `signOut`
- `hydrate`
- `syncWorkspace`
- token clearing on sign-out
- session-expired event integration

**Step 2: Run test to verify it fails**

Run:

```powershell
node --test tests/portal-auth-parity.test.mjs
```

Expected: FAIL because the store contract is still missing.

**Step 3: Write minimal implementation**

Implement `usePortalAuthStore` backed by:

- `loginPortalUser`
- `registerPortalUser`
- `getPortalMe`
- `getPortalWorkspace`
- `getPortalDashboard`
- `readPortalSessionToken`
- `persistPortalSessionToken`
- `clearPortalSessionToken`
- `onPortalSessionExpired`

Move `PortalProductApp` to consume the store instead of duplicating auth lifecycle state locally.

**Step 4: Run test to verify it passes**

Run:

```powershell
node --test tests/portal-auth-parity.test.mjs
pnpm --dir apps/sdkwork-router-portal typecheck
```

Expected: PASS.

**Step 5: Commit**

```bash
git commit apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/store/usePortalAuthStore.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/app/PortalProductApp.tsx -m "feat: add shared portal auth store"
```

### Task 3: Convert Auth Routing To Claw-Style Route Semantics

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/routePaths.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/routeManifest.ts`

**Step 1: Write the failing test**

Require:

- `/login`
- `/register`
- `/forgot-password`
- `/auth` redirect
- redirect restore for protected routes

**Step 2: Run test to verify it fails**

Run:

```powershell
node --test tests/portal-auth-parity.test.mjs tests/portal-shell-parity.test.mjs
```

Expected: FAIL because the route surface is still incomplete.

**Step 3: Write minimal implementation**

Update route helpers and `AppRoutes.tsx` so:

- auth routes render through one auth page contract
- `/auth` redirects to `/login`
- protected routes redirect unauthenticated users with `?redirect=...`
- authenticated users visiting auth routes are redirected to dashboard or the resolved target

**Step 4: Run test to verify it passes**

Run:

```powershell
node --test tests/portal-auth-parity.test.mjs tests/portal-shell-parity.test.mjs
pnpm --dir apps/sdkwork-router-portal typecheck
```

Expected: PASS.

**Step 5: Commit**

```bash
git commit apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/routePaths.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/routeManifest.ts -m "feat: align portal auth routes with claw shell"
```

### Task 4: Rebuild The Auth Package Around Router-Driven Mode Resolution

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-auth/src/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-auth/src/types/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-auth/src/components/index.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-auth/src/pages/AuthPage.tsx`
- Delete or reduce: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-auth/src/pages/index.tsx`

**Step 1: Write the failing test**

Require:

- auth mode is derived from pathname
- login and register submit through shared store actions
- forgot-password renders as a distinct route mode
- auth mode switching uses route navigation, not callback props

**Step 2: Run test to verify it fails**

Run:

```powershell
node --test tests/portal-auth-parity.test.mjs
```

Expected: FAIL.

**Step 3: Write minimal implementation**

Build a new `AuthPage.tsx` that:

- follows the `claw-studio` route-driven auth page pattern
- uses portal product copy
- calls shared auth store `signIn` and `register`
- preserves `redirect` query behavior
- exposes a presentational forgot-password mode

**Step 4: Run test to verify it passes**

Run:

```powershell
node --test tests/portal-auth-parity.test.mjs
pnpm --dir apps/sdkwork-router-portal typecheck
```

Expected: PASS.

**Step 5: Commit**

```bash
git commit apps/sdkwork-router-portal/packages/sdkwork-router-portal-auth/src/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-auth/src/types/index.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-auth/src/components/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-auth/src/pages/AuthPage.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-auth/src/pages/index.tsx -m "feat: rebuild portal auth page around claw-style routing"
```

### Task 5: Wire Sidebar Footer And Shell To Shared Auth State

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx`

**Step 1: Write the failing test**

Require:

- sidebar footer uses shared auth state
- unauthenticated user control routes to `/login`
- sign-out routes to `/login`
- workspace identity still renders when available

**Step 2: Run test to verify it fails**

Run:

```powershell
node --test tests/portal-shell-parity.test.mjs tests/portal-auth-parity.test.mjs
```

Expected: FAIL.

**Step 3: Write minimal implementation**

Update the profile dock and sidebar so shell identity and logout behavior come from `usePortalAuthStore`, not only from shell props.

**Step 4: Run test to verify it passes**

Run:

```powershell
node --test tests/portal-shell-parity.test.mjs tests/portal-auth-parity.test.mjs
pnpm --dir apps/sdkwork-router-portal typecheck
```

Expected: PASS.

**Step 5: Commit**

```bash
git commit apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx -m "feat: wire portal shell to shared auth state"
```

### Task 6: Run Full Portal Verification And Close Remaining Gaps

**Files:**
- Modify: any touched portal files if verification exposes gaps
- Test: `apps/sdkwork-router-portal/tests/*.mjs`

**Step 1: Run the full test suite**

Run:

```powershell
node --test tests/*.mjs
```

Expected: PASS.

**Step 2: Run full typecheck and build**

Run:

```powershell
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
```

Expected: PASS.

**Step 3: Fix the smallest remaining gaps**

Only adjust code needed to satisfy:

- auth parity
- shell parity
- theme parity
- desktop shell integrity

**Step 4: Re-run full verification**

Run:

```powershell
node --test tests/*.mjs
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
```

Expected: PASS with fresh evidence.

**Step 5: Commit**

```bash
git commit apps/sdkwork-router-portal docs/plans/2026-03-19-sdkwork-router-portal-claw-auth-shell-design.md docs/plans/2026-03-19-sdkwork-router-portal-claw-auth-shell.md -m "feat: align portal auth shell with claw studio"
```
