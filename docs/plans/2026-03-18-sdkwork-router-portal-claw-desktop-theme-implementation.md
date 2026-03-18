# SDKWork Router Portal Claw Desktop Theme Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Align `apps/sdkwork-router-portal` with `claw-studio` shell, theme, sidebar, and desktop behaviors, including a real Tauri host, a right-aligned `WindowControls` component, and content components that fully follow theme changes.

**Architecture:** Keep the existing portal business packages and browser router, but make `sdkwork-router-portal-core` the claw-style shell boundary. Normalize persisted shell defaults and theme tokens first, then rebuild the header, sidebar, config center, and desktop host, then sweep shared primitives and portal pages so every visible surface is driven by the shared portal token contract.

**Tech Stack:** React 19, TypeScript, Vite, Tailwind CSS v4, Radix UI, React Router DOM, Zustand persist, Recharts, Tauri v2, Rust

---

### Task 1: Strengthen the shell parity tests before implementation

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-navigation-polish.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-form-ux-polish.test.mjs`

**Step 1: Write the failing assertions**

Add assertions that require:

- default `themeMode` to be `system`
- default `themeColor` to be `lobster`
- a dedicated `WindowControls` component in the portal shell
- sidebar footer avatar plus settings icon semantics
- a `src-tauri` desktop host directory
- theme token usage in shared components instead of hard-coded shell colors

Example assertions:

```js
assert.match(storeSource, /themeMode:\s*'system'/);
assert.match(storeSource, /themeColor:\s*'lobster'/);
assert.match(headerSource, /WindowControls/);
assert.match(sidebarSource, /Settings/);
assert.match(sidebarSource, /avatar|display_name|userInitials/);
assert.equal(existsSync(path.join(appRoot, 'src-tauri', 'tauri.conf.json')), true);
```

**Step 2: Run the tests and confirm they fail**

Run:

```powershell
node --test tests/portal-shell-parity.test.mjs tests/portal-theme-config.test.mjs tests/portal-navigation-polish.test.mjs tests/portal-form-ux-polish.test.mjs
```

Expected:

- FAIL because defaults still differ, `src-tauri` does not exist, and shell parity is incomplete

**Step 3: Stop after the failing evidence**

Do not implement in this task.

**Step 4: Commit**

```bash
git commit apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs apps/sdkwork-router-portal/tests/portal-navigation-polish.test.mjs apps/sdkwork-router-portal/tests/portal-form-ux-polish.test.mjs -m "test: define portal claw desktop parity"
```

### Task 2: Normalize shell defaults and desktop-safe shell preferences

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/store/usePortalShellStore.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/lib/portalPreferences.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/providers/ThemeManager.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/lib/desktop.ts`

**Step 1: Implement the minimal preference changes**

Set the shell defaults to:

```ts
themeMode: 'system',
themeColor: 'lobster',
```

Keep the persisted preference key stable unless a migration becomes necessary. Keep `data-theme` plus `.dark` as the only theme application mechanism.

In `desktop.ts`, keep browser-safe fallbacks and expose helpers for:

```ts
isTauriDesktop();
minimizeWindow();
maximizeWindow();
closeWindow();
```

**Step 2: Run the parity tests**

Run:

```powershell
node --test tests/portal-shell-parity.test.mjs tests/portal-theme-config.test.mjs
```

Expected:

- some assertions now pass
- tests still fail because shell components and host files are not complete

**Step 3: Run portal typecheck**

Run:

```powershell
pnpm --dir apps/sdkwork-router-portal typecheck
```

Expected:

- PASS

**Step 4: Commit**

```bash
git commit apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/store/usePortalShellStore.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/lib/portalPreferences.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/providers/ThemeManager.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/lib/desktop.ts -m "feat: align portal shell preference defaults"
```

### Task 3: Add the real Tauri host scaffold for the portal app

**Files:**
- Modify: `apps/sdkwork-router-portal/package.json`
- Create: `apps/sdkwork-router-portal/src-tauri/build.rs`
- Create: `apps/sdkwork-router-portal/src-tauri/Cargo.toml`
- Create: `apps/sdkwork-router-portal/src-tauri/tauri.conf.json`
- Create: `apps/sdkwork-router-portal/src-tauri/src/main.rs`

**Step 1: Use the admin app host as the local reference**

Mirror the minimal host shape from:

- `apps/sdkwork-router-admin/src-tauri/build.rs`
- `apps/sdkwork-router-admin/src-tauri/Cargo.toml`
- `apps/sdkwork-router-admin/src-tauri/tauri.conf.json`
- `apps/sdkwork-router-admin/src-tauri/src/main.rs`

Add root scripts:

```json
"tauri:dev": "tauri dev",
"tauri:build": "tauri build"
```

and the matching dev dependency:

```json
"@tauri-apps/cli": "^2.2.5"
```

**Step 2: Run the structural tests**

Run:

```powershell
node --test tests/portal-shell-parity.test.mjs tests/portal-theme-config.test.mjs
```

Expected:

- the `src-tauri` existence assertions pass
- shell component assertions may still fail

**Step 3: Run portal typecheck**

Run:

```powershell
pnpm --dir apps/sdkwork-router-portal typecheck
```

Expected:

- PASS

**Step 4: Commit**

```bash
git commit apps/sdkwork-router-portal/package.json apps/sdkwork-router-portal/src-tauri -m "feat: add portal tauri host scaffold"
```

### Task 4: Rebuild the main shell layout and desktop header

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/AppHeader.tsx`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/WindowControls.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/ShellStatus.tsx`

**Step 1: Implement `WindowControls` as a dedicated shell component**

Create a component shape like:

```tsx
export function WindowControls() {
  return (
    <div data-tauri-drag-region="false">
      <button type="button" onClick={() => void minimizeWindow()} />
      <button type="button" onClick={() => void maximizeWindow()} />
      <button type="button" onClick={() => void closeWindow()} />
    </div>
  );
}
```

Render it from `AppHeader.tsx` only when `isTauriDesktop()` is true.

**Step 2: Rework the header into a desktop shell**

Ensure:

- the left area is the drag region
- interactive controls opt out with `data-tauri-drag-region="false"`
- the center workspace surface feels like a desktop title bar slot
- the right side owns the config trigger plus `WindowControls`

**Step 3: Rework the main layout**

Keep:

- header on top
- sidebar on the left
- right content region as the primary work surface

Do not let the shell status panel become a second header.

**Step 4: Run shell parity tests and typecheck**

Run:

```powershell
node --test tests/portal-shell-parity.test.mjs tests/portal-navigation-polish.test.mjs
pnpm --dir apps/sdkwork-router-portal typecheck
```

Expected:

- PASS for header, drag-region, and `WindowControls` assertions
- PASS for TypeScript

**Step 5: Commit**

```bash
git commit apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/AppHeader.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/WindowControls.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/ShellStatus.tsx -m "feat: align portal desktop header and shell layout"
```

### Task 5: Rebuild the sidebar and configuration center to match the approved shell

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx`
- Modify: `apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs`

**Step 1: Implement the sidebar footer semantics**

Ensure the sidebar bottom area contains:

- avatar or initials surface
- user identity copy in expanded mode
- settings icon button
- sign-out button if the existing flow requires it

Collapsed mode must still expose avatar plus settings affordances.

**Step 2: Keep desktop sidebar behavior**

Implement or preserve:

- click collapse and expand
- hover edge affordance
- drag resize handle
- grouped nav sections
- active-route indicator

**Step 3: Align the configuration center**

Keep only shell-level controls:

- theme mode
- theme color
- sidebar navigation visibility

Use the shared dialog and token system rather than page-local surfaces.

**Step 4: Run shell parity and theme tests**

Run:

```powershell
node --test tests/portal-shell-parity.test.mjs tests/portal-theme-config.test.mjs
```

Expected:

- PASS

**Step 5: Commit**

```bash
git commit apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs apps/sdkwork-router-portal/tests/portal-theme-config.test.mjs -m "feat: align portal sidebar and config center"
```

### Task 6: Align the shared theme token contract and commons surfaces

**Files:**
- Modify: `apps/sdkwork-router-portal/src/theme.css`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`

**Step 1: Rebuild the token contract**

Keep the six approved theme colors and define:

- shell background
- content background
- sidebar background
- border colors
- text hierarchy
- overlay surfaces
- chart surfaces
- shadow levels

Also bring in the `claw-studio` scrollbar treatment where it fits the portal app.

**Step 2: Make commons the enforcement layer**

Update shared primitives so that:

- `Surface`
- `MetricCard`
- `Dialog`
- `Tabs`
- `Input`
- `Select`
- `Checkbox`
- `DataTable`
- `Pill`

all consume shared semantic tokens.

Example direction:

```tsx
const portalSurface = 'bg-[var(--portal-surface-background)]';
const portalText = 'text-[var(--portal-text-primary)]';
```

**Step 3: Run theme tests and typecheck**

Run:

```powershell
node --test tests/portal-theme-config.test.mjs tests/portal-form-ux-polish.test.mjs
pnpm --dir apps/sdkwork-router-portal typecheck
```

Expected:

- PASS

**Step 4: Commit**

```bash
git commit apps/sdkwork-router-portal/src/theme.css apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx -m "feat: unify portal theme tokens and shared surfaces"
```

### Task 7: Sweep the portal business pages so content fully follows theme changes

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/components/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-routing/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-usage/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-usage/src/components/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-user/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-account/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/tests/portal-dashboard-analytics.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-routing-polish.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-navigation-polish.test.mjs`

**Step 1: Update the tests first**

Add or adjust assertions so they require:

- right-pane content structure
- token-driven cards and charts
- no regression to hard-coded shell colors for key surfaces
- shell-aligned copy for module navigation and actions

**Step 2: Run the page tests and confirm they fail**

Run:

```powershell
node --test tests/portal-dashboard-analytics.test.mjs tests/portal-routing-polish.test.mjs tests/portal-navigation-polish.test.mjs
```

Expected:

- FAIL where pages still rely on pre-parity surface assumptions

**Step 3: Make the page surfaces token-driven**

Replace page-local shell surfaces with shared primitives or shared semantic colors.

For charts, ensure:

- grid colors use `--portal-chart-grid`
- axes use `--portal-chart-axis`
- tooltip surfaces use `--portal-chart-tooltip-background`

**Step 4: Run the updated page tests**

Run:

```powershell
node --test tests/portal-dashboard-analytics.test.mjs tests/portal-routing-polish.test.mjs tests/portal-navigation-polish.test.mjs
```

Expected:

- PASS

**Step 5: Commit**

```bash
git commit apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/components/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-routing/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-usage/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-usage/src/components/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-user/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-account/src/pages/index.tsx apps/sdkwork-router-portal/tests/portal-dashboard-analytics.test.mjs apps/sdkwork-router-portal/tests/portal-routing-polish.test.mjs apps/sdkwork-router-portal/tests/portal-navigation-polish.test.mjs -m "feat: make portal content surfaces follow shell themes"
```

### Task 8: Run the full verification pass before calling the work complete

**Files:**
- None

**Step 1: Run the shell and theme test suite**

Run:

```powershell
node --test tests/portal-shell-parity.test.mjs tests/portal-theme-config.test.mjs tests/portal-navigation-polish.test.mjs tests/portal-form-ux-polish.test.mjs tests/portal-dashboard-analytics.test.mjs tests/portal-routing-polish.test.mjs
```

Expected:

- PASS

**Step 2: Run portal typecheck**

Run:

```powershell
pnpm --dir apps/sdkwork-router-portal typecheck
```

Expected:

- PASS

**Step 3: Run portal build**

Run:

```powershell
pnpm --dir apps/sdkwork-router-portal build
```

Expected:

- PASS

**Step 4: Run the Tauri desktop smoke check**

Run:

```powershell
pnpm --dir apps/sdkwork-router-portal tauri:build
```

Expected:

- PASS, or a platform-specific packaging prerequisite error that is unrelated to the portal shell code itself

**Step 5: Commit**

```bash
git commit -m "feat: complete portal claw desktop theme parity"
```
