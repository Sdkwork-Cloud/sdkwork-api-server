# SDKWork Router Portal Claw Shell Alignment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rebuild `apps/sdkwork-router-portal` so its shell architecture, theme system, settings center, and sidebar behavior align with `claw-studio` while preserving the current portal business features.

**Architecture:** Refactor `sdkwork-router-portal-core` into a shell-style package with providers, router, layout, shell components, and persisted UI preferences. Move the portal from hash navigation to a real `/portal/*` browser router, introduce a Claw-like theme manager and configuration center, then restyle portal business pages to sit inside the new shell contract.

**Tech Stack:** React 19, TypeScript, Vite, Tailwind CSS v4, Radix UI, Recharts, `react-router-dom`, `zustand`

---

### Task 1: Lock The Shell Architecture In Tests

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-architecture.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-admin-ui-foundation.test.mjs`
- Create: `apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs`
- Create: `apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs`

**Step 1: Write the failing shell architecture tests**

Add assertions that require:

- `sdkwork-router-portal-core` to expose `application/app`, `application/layouts`, `application/providers`, `application/router`, `components`, and `store`
- `react-router-dom` and `zustand` dependencies in the app stack
- a `ThemeManager` that writes `data-theme`
- a `ConfigCenter` shell component
- a `Sidebar` with collapse and resize affordances

Example assertions:

```js
assert.equal(existsSync(path.join(appRoot, 'packages', 'sdkwork-router-portal-core', 'src', 'application', 'layouts', 'MainLayout.tsx')), true);
assert.match(corePackage, /react-router-dom/);
assert.match(shellThemeTestSource, /data-theme/);
assert.match(shellParitySource, /collapseSidebar|toggleSidebar/);
```

**Step 2: Run the new tests to verify they fail**

Run:

```powershell
node --test tests/portal-architecture.test.mjs tests/portal-admin-ui-foundation.test.mjs tests/portal-shell-parity.test.mjs tests/portal-theme-config.test.mjs
```

Expected:

- FAIL because the new shell files and parity strings do not exist yet

**Step 3: Do not implement yet**

Stop after confirming the expected failures.

**Step 4: Commit checkpoint**

```bash
git commit apps/sdkwork-router-portal/tests/portal-architecture.test.mjs apps/sdkwork-router-portal/tests/portal-admin-ui-foundation.test.mjs apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs -m "test: define portal claw shell parity"
```

### Task 2: Build The Core Shell, Router, And Theme Manager

**Files:**
- Modify: `apps/sdkwork-router-portal/package.json`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/package.json`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/routes.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/app/PortalProductApp.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/providers/AppProviders.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/providers/ThemeManager.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/routeManifest.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/routePaths.ts`

**Step 1: Write the smallest implementation needed for shell routing**

Implement:

- `react-router-dom` BrowserRouter setup with `basename="/portal"`
- a shell route manifest that maps portal route keys to browser paths
- lazy business page loading through route elements rather than hash checks
- `ThemeManager` that writes `data-theme` and `dark` class onto `document.documentElement`

Example shape:

```tsx
export function AppProviders({ children }: { children: ReactNode }) {
  return (
    <BrowserRouter basename="/portal">
      <ThemeManager />
      {children}
    </BrowserRouter>
  );
}
```

**Step 2: Run the shell architecture tests**

Run:

```powershell
node --test tests/portal-architecture.test.mjs tests/portal-admin-ui-foundation.test.mjs tests/portal-shell-parity.test.mjs tests/portal-theme-config.test.mjs
```

Expected:

- PASS for file structure and basic shell/theme architecture checks

**Step 3: Run typecheck for the portal app**

Run:

```powershell
pnpm --dir apps/sdkwork-router-portal typecheck
```

Expected:

- PASS with no TypeScript errors

**Step 4: Commit checkpoint**

```bash
git commit apps/sdkwork-router-portal/package.json apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/package.json apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/routes.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application -m "feat: add portal claw-aligned shell foundation"
```

### Task 3: Add Persisted Portal Preferences, Sidebar Parity, And Config Center

**Files:**
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/store/usePortalShellStore.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/lib/portalPreferences.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/AppHeader.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/ShellStatus.tsx`
- Modify: `apps/sdkwork-router-portal/src/theme.css`

**Step 1: Write the failing sidebar and config tests**

Extend the new parity tests so they require:

- a collapsed sidebar width contract
- a resize handle
- a settings trigger
- theme color options like `tech-blue`, `lobster`, `green-tech`, `zinc`, `violet`, `rose`
- `hiddenSidebarItems`, `sidebarWidth`, and `isSidebarCollapsed` persistence keys

**Step 2: Run the tests to verify the new assertions fail**

Run:

```powershell
node --test tests/portal-shell-parity.test.mjs tests/portal-theme-config.test.mjs
```

Expected:

- FAIL because the store and config center are still incomplete

**Step 3: Implement the portal shell store and shell components**

Use `zustand` persist middleware to store:

- `themeMode`
- `themeColor`
- `isSidebarCollapsed`
- `sidebarWidth`
- `hiddenSidebarItems`

Implement:

- Claw-like dark shell sidebar
- click collapse and expand control
- drag resize handle on desktop
- settings/config center dialog or panel
- shell header with route title and workspace status
- theme variable palettes in `src/theme.css`

Example store shape:

```ts
export const usePortalShellStore = create<PortalShellState>()(
  persist(
    (set) => ({
      isSidebarCollapsed: false,
      sidebarWidth: 252,
      themeMode: 'system',
      themeColor: 'tech-blue',
      hiddenSidebarItems: [],
    }),
    { name: 'sdkwork-router-portal.preferences.v1' },
  ),
);
```

**Step 4: Run shell parity tests and typecheck**

Run:

```powershell
node --test tests/portal-shell-parity.test.mjs tests/portal-theme-config.test.mjs
pnpm --dir apps/sdkwork-router-portal typecheck
```

Expected:

- PASS for sidebar/config/theme assertions
- PASS for TypeScript

**Step 5: Commit checkpoint**

```bash
git commit apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/store apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/lib apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components apps/sdkwork-router-portal/src/theme.css -m "feat: add portal sidebar and config center parity"
```

### Task 4: Restyle Portal Business Pages To Live Inside The New Shell

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-routing/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-usage/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-user/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-account/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/tests/portal-navigation-polish.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-mode-polish.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-form-ux-polish.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-dashboard-analytics.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-routing-polish.test.mjs`

**Step 1: Update the page-facing tests first**

Adjust or extend the page tests so they enforce:

- shell-aligned terminology and structure
- shell header and module cards still present
- dialogs, tabs, charts, and analytics survive the shell migration
- pages render as right-side work surfaces rather than old `portalx-*` shell blocks

**Step 2: Run the updated page tests to watch them fail**

Run:

```powershell
node --test tests/portal-navigation-polish.test.mjs tests/portal-mode-polish.test.mjs tests/portal-form-ux-polish.test.mjs tests/portal-dashboard-analytics.test.mjs tests/portal-routing-polish.test.mjs
```

Expected:

- FAIL where page wording and structure still reflect the old shell

**Step 3: Implement the page restyle and composition cleanup**

Update shared primitives and page composition so that:

- surfaces inherit the new shell theme cleanly
- metric cards, tables, charts, tabs, dialogs, and empty states share one visual contract
- pages look like product workbench screens inside the shell instead of standalone microsites

**Step 4: Run the targeted page tests and a portal build**

Run:

```powershell
node --test tests/portal-navigation-polish.test.mjs tests/portal-mode-polish.test.mjs tests/portal-form-ux-polish.test.mjs tests/portal-dashboard-analytics.test.mjs tests/portal-routing-polish.test.mjs
pnpm --dir apps/sdkwork-router-portal build
```

Expected:

- PASS for page polish tests
- PASS for production build

**Step 5: Commit checkpoint**

```bash
git commit apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-routing/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-usage/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-user/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-account/src/pages/index.tsx apps/sdkwork-router-portal/tests/portal-navigation-polish.test.mjs apps/sdkwork-router-portal/tests/portal-mode-polish.test.mjs apps/sdkwork-router-portal/tests/portal-form-ux-polish.test.mjs apps/sdkwork-router-portal/tests/portal-dashboard-analytics.test.mjs apps/sdkwork-router-portal/tests/portal-routing-polish.test.mjs -m "feat: align portal pages with claw shell"
```

### Task 5: Run Full Portal Regression And Final Polish

**Files:**
- Modify: any portal app files that still fail verification
- Test: `apps/sdkwork-router-portal/tests/*.mjs`

**Step 1: Run the full portal test suite**

Run:

```powershell
node --test tests/*.mjs
```

Expected:

- PASS across the portal Node test suite

**Step 2: Run full typecheck and build again**

Run:

```powershell
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
```

Expected:

- PASS for both commands

**Step 3: Fix any remaining shell parity or polish gaps**

Only make the smallest changes required to get:

- theme parity
- sidebar parity
- config center consistency
- right-side content area correctness
- page visual consistency

**Step 4: Re-run the full verification after every fix**

Run:

```powershell
node --test tests/*.mjs
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
```

Expected:

- PASS with clean exit codes

**Step 5: Final commit**

```bash
git commit apps/sdkwork-router-portal -m "feat: align portal shell with claw studio"
```
