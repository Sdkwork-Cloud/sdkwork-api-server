# Router Admin Claw Auth Shell Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Align `apps/sdkwork-router-admin` with `claw-studio` for auth route shape, login presentation, shell composition, sidebar behavior, and theme/settings parity while keeping real admin backend authentication.

**Architecture:** Keep the current admin package boundaries, extend the route/auth contract in `core` and `shell`, rebuild the auth surface to match the claw product language, and refine shell/settings styling so the admin app uses the same visual and interaction grammar as the reference product.

**Tech Stack:** React 19, TypeScript 5.8, Vite 7, React Router 7, Zustand 5, Motion 12, custom shell CSS

---

### Task 1: Lock The Parity Contract In Tests

**Files:**
- Modify: `apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`
- Modify: `apps/sdkwork-router-admin/tests/admin-shell-parity.test.mjs`

**Step 1: Write the failing test**

Add assertions for:

- `AUTH`, `REGISTER`, and `FORGOT_PASSWORD` route constants
- auth routing for `/auth`, `/register`, and `/forgot-password`
- claw-style auth composition markers such as the QR/auth split and admin-specific route guidance

**Step 2: Run test to verify it fails**

Run: `node --test tests/admin-architecture.test.mjs tests/admin-shell-parity.test.mjs`
Expected: FAIL because the new auth routes and parity markers are not all present yet.

**Step 3: Write minimal implementation**

Do not implement yet. This task only defines the red test baseline.

**Step 4: Run test to verify it still fails for the expected reason**

Run: `node --test tests/admin-architecture.test.mjs tests/admin-shell-parity.test.mjs`
Expected: FAIL with missing auth-route or auth-surface assertions.

### Task 2: Add Auth Route Parity In Core And Shell

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routePaths.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/application/layouts/MainLayout.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/AppRoutes.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/MainLayout.tsx`

**Step 1: Write the failing test**

Use the Task 1 assertions as the failing test for new route constants and auth-route handling.

**Step 2: Run test to verify it fails**

Run: `node --test tests/admin-architecture.test.mjs tests/admin-shell-parity.test.mjs`
Expected: FAIL because the route constants and auth-route branches are incomplete.

**Step 3: Write minimal implementation**

Implement:

- `AUTH`, `REGISTER`, and `FORGOT_PASSWORD` in `ADMIN_ROUTE_PATHS`
- auth-route recognition in both layout entrypoints
- route handling for `/auth`, `/register`, and `/forgot-password`

Keep protected business routes unchanged.

**Step 4: Run test to verify it passes**

Run: `node --test tests/admin-architecture.test.mjs tests/admin-shell-parity.test.mjs`
Expected: PASS for route-shape assertions, with remaining auth-visual assertions still potentially failing.

### Task 3: Rebuild The Admin Auth Surface To Match Claw

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-auth/package.json`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-auth/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/styles/index.css`

**Step 1: Write the failing test**

Extend the parity tests to assert the presence of:

- mode resolution for login/register/forgot
- QR/complementary panel copy
- icon-led email/password fields
- admin-specific non-login guidance

**Step 2: Run test to verify it fails**

Run: `node --test tests/admin-architecture.test.mjs tests/admin-shell-parity.test.mjs`
Expected: FAIL because the current admin login page is still the old two-card custom surface.

**Step 3: Write minimal implementation**

Refactor `sdkwork-router-admin-auth` into a route-aware auth page that:

- mirrors the `claw-studio` auth structure
- submits only login mode to `onLogin`
- provides truthful admin guidance for register and forgot modes
- keeps loading and backend status feedback visible

Add only the package dependencies required by the new auth page.

**Step 4: Run test to verify it passes**

Run: `node --test tests/admin-architecture.test.mjs tests/admin-shell-parity.test.mjs`
Expected: PASS for auth-route and auth-surface assertions.

### Task 4: Tighten Header, Sidebar, And Content Canvas Parity

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/application/layouts/MainLayout.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/components/AppHeader.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/styles/index.css`

**Step 1: Write the failing test**

Use the existing shell parity tests and add any missing assertions for:

- auth atmosphere wrapper
- right-canvas content surface
- richer bottom account controls
- claw-like header balance and search/settings affordance markers

**Step 2: Run test to verify it fails**

Run: `node --test tests/admin-shell-parity.test.mjs`
Expected: FAIL on any new shell parity markers that are not present yet.

**Step 3: Write minimal implementation**

Refine the shell so that:

- auth routes and shell routes share the same ambient background logic
- header reads closer to the reference shell
- sidebar maintains collapse, resize, active-rail, and bottom account affordances
- right canvas is visually isolated as the only content region

**Step 4: Run test to verify it passes**

Run: `node --test tests/admin-shell-parity.test.mjs`
Expected: PASS.

### Task 5: Refine Settings Center Theme And Sidebar Continuity

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/AppearanceSettings.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/NavigationSettings.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-settings/src/Settings.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/styles/index.css`

**Step 1: Write the failing test**

Add assertions for settings copy and preview markers that tie theme, sidebar posture, and right-canvas continuity together.

**Step 2: Run test to verify it fails**

Run: `node --test tests/admin-shell-parity.test.mjs tests/admin-product-experience.test.mjs`
Expected: FAIL because the settings wording and preview contract are not fully aligned yet.

**Step 3: Write minimal implementation**

Refine settings content and styling so:

- all six theme presets remain visible
- sidebar preview and shell continuity language remain explicit
- settings visuals align more closely with the claw shell language

**Step 4: Run test to verify it passes**

Run: `node --test tests/admin-shell-parity.test.mjs tests/admin-product-experience.test.mjs`
Expected: PASS.

### Task 6: Full Verification

**Files:**
- No code changes required

**Step 1: Run static contract tests**

Run: `node --test tests/*.mjs`
Expected: PASS

**Step 2: Run typecheck**

Run: `pnpm --dir apps/sdkwork-router-admin typecheck`
Expected: PASS

**Step 3: Run production build**

Run: `pnpm --dir apps/sdkwork-router-admin build`
Expected: PASS

**Step 4: Review results**

Confirm that:

- auth route parity is in place
- login remains backed by the real admin API
- shell/sidebar/theme/settings behavior remains intact

**Step 5: Commit**

Only if the user later asks for it:

```bash
git add docs/plans/2026-03-19-router-admin-claw-auth-shell-design.md docs/plans/2026-03-19-router-admin-claw-auth-shell-implementation.md apps/sdkwork-router-admin
git commit -m "feat: align router admin auth shell with claw studio"
```
