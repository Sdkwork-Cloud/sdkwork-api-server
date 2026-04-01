# Router Admin UI Standardization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild `sdkwork-router-admin` on top of `@sdkwork/ui-pc-react` so the shell, auth flow, settings center, and every business page use one shared PC UI framework and the legacy `adminx-*` presentation system is removed.

**Architecture:** The app keeps its current route graph, workbench store, and feature packages, but the presentation layer is rewritten to consume `@sdkwork/ui-pc-react` directly. The implementation removes the old local UI framework, rewires the app root and shell to shared shell patterns, then rebuilds feature pages around shared workbench, settings, form, overlay, and data-display patterns.

**Tech Stack:** React 19, TypeScript, Vite 7, React Router 7, Tauri host bootstrap, `@sdkwork/ui-pc-react`, existing admin feature packages, Node `node:test` architecture checks

---

## File Structure

### Root and dependency wiring

- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tsconfig.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\vite.config.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\src\main.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\src\App.tsx`

### Shell and shared app composition

- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\index.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\application\providers\AppProviders.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\application\providers\ThemeManager.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\application\layouts\MainLayout.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\application\router\AppRoutes.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\components\AppHeader.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\components\Sidebar.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\components\ShellStatus.tsx`
- Delete: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\styles\index.css`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\styles\shell-host.css`

### Common i18n and legacy UI removal

- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-commons\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-commons\package.json`
- Delete: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-commons\src\index.tsx` after consumers are migrated
- Delete: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-commons\package.json` after dependency removal

### Feature pages

- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-auth\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-overview\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-users\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-tenants\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-coupons\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-catalog\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-traffic\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-operations\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-settings\src\Settings.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-settings\src\GeneralSettings.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-settings\src\AppearanceSettings.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-settings\src\NavigationSettings.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-settings\src\WorkspaceSettings.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-settings\src\Shared.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-apirouter\src\pages\GatewayAccessPage.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-apirouter\src\pages\GatewayRoutesPage.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-apirouter\src\pages\GatewayModelMappingsPage.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-apirouter\src\pages\GatewayUsagePage.tsx`

### Package dependency cleanup

- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-auth\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-overview\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-users\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-tenants\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-coupons\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-catalog\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-traffic\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-operations\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-settings\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-apirouter\package.json`

### Tests

- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-architecture.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-product-experience.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-shell-parity.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-crud-ux.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-claw-foundation-parity.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-table-polish.test.mjs`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-ui-framework-adoption.test.mjs`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-page-patterns.test.mjs`

### Documentation

- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\README.md`

## Task 1: Wire the shared UI framework into the app root

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tsconfig.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\vite.config.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\src\main.tsx`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-ui-framework-adoption.test.mjs`

- [ ] **Step 1: Write the failing architecture test**

```javascript
test('app root imports sdkwork ui framework and stylesheet directly', () => {
  const main = read('src/main.tsx');
  const packageJson = readJson('package.json');

  assert.match(main, /@sdkwork\/ui-pc-react\/styles\.css/);
  assert.ok(packageJson.dependencies['@sdkwork/ui-pc-react']);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/admin-ui-framework-adoption.test.mjs`
Expected: FAIL because the root entry and dependencies do not yet reference `@sdkwork/ui-pc-react`

- [ ] **Step 3: Write minimal implementation**

```json
{
  "dependencies": {
    "@sdkwork/ui-pc-react": "file:../../../sdkwork-ui/sdkwork-ui-pc-react"
  }
}
```

```ts
import '@sdkwork/ui-pc-react/styles.css';
```

Add matching TypeScript and Vite resolution so app imports use the real package name.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/admin-ui-framework-adoption.test.mjs`
Expected: PASS

- [ ] **Step 5: Run type verification**

Run: `pnpm typecheck`
Expected: PASS or only unrelated pre-existing failures outside this task scope

- [ ] **Step 6: Commit**

```bash
git add package.json tsconfig.json vite.config.ts src/main.tsx tests/admin-ui-framework-adoption.test.mjs
git commit -m "feat: wire router admin to sdkwork ui framework"
```

## Task 2: Rebuild providers and authenticated shell on shared UI patterns

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\application\providers\AppProviders.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\application\providers\ThemeManager.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\application\layouts\MainLayout.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\components\AppHeader.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\components\Sidebar.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\components\ShellStatus.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\application\router\AppRoutes.tsx`
- Delete: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\styles\index.css`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-shell\src\styles\shell-host.css`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-shell-parity.test.mjs`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-page-patterns.test.mjs`

- [ ] **Step 1: Write the failing shell tests**

```javascript
test('shell uses sdkwork app shell and workspace patterns instead of adminx classes', () => {
  const layout = read('packages/sdkwork-router-admin-shell/src/application/layouts/MainLayout.tsx');

  assert.match(layout, /AppShell|DesktopShellFrame|WorkspaceScaffold/);
  assert.doesNotMatch(layout, /adminx-shell/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/admin-shell-parity.test.mjs tests/admin-page-patterns.test.mjs`
Expected: FAIL because the shell still depends on `adminx-*`

- [ ] **Step 3: Write minimal implementation**

```tsx
return (
  <AppShell
    header={<AdminHeader />}
    sidebar={<AdminSidebar />}
    content={<AppRoutes />}
  />
);
```

Move host-only rules into `shell-host.css` and delete the legacy shell stylesheet when no feature depends on it.

- [ ] **Step 4: Run tests to verify they pass**

Run: `node --test tests/admin-shell-parity.test.mjs tests/admin-page-patterns.test.mjs`
Expected: PASS

- [ ] **Step 5: Run type verification**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add packages/sdkwork-router-admin-shell tests/admin-shell-parity.test.mjs tests/admin-page-patterns.test.mjs
git commit -m "feat: rebuild router admin shell on shared sdkwork patterns"
```

## Task 3: Rebuild auth and settings around shared forms and settings patterns

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-auth\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-settings\src\Settings.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-settings\src\GeneralSettings.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-settings\src\AppearanceSettings.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-settings\src\NavigationSettings.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-settings\src\WorkspaceSettings.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-settings\src\Shared.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-product-experience.test.mjs`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-product-experience.test.mjs`

- [ ] **Step 1: Write the failing auth/settings tests**

```javascript
test('auth uses shared cards, form controls, and buttons from sdkwork ui', () => {
  const auth = read('packages/sdkwork-router-admin-auth/src/index.tsx');

  assert.match(auth, /@sdkwork\/ui-pc-react/);
  assert.doesNotMatch(auth, /adminx-auth-/);
});

test('settings uses SettingsCenter instead of local settings scaffolding', () => {
  const settings = read('packages/sdkwork-router-admin-settings/src/Settings.tsx');

  assert.match(settings, /SettingsCenter/);
  assert.doesNotMatch(settings, /adminx-/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/admin-product-experience.test.mjs`
Expected: FAIL because auth and settings still use legacy local UI

- [ ] **Step 3: Write minimal implementation**

```tsx
<SettingsCenter
  title={t('Settings center')}
  sections={sections}
  activeItem={activeItem}
  onActiveItemChange={setActiveItem}
>
  {activePanel}
</SettingsCenter>
```

```tsx
<Card>
  <Form>
    <FormField ... />
  </Form>
</Card>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/admin-product-experience.test.mjs`
Expected: PASS

- [ ] **Step 5: Run type verification**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add packages/sdkwork-router-admin-auth packages/sdkwork-router-admin-settings tests/admin-product-experience.test.mjs
git commit -m "feat: rebuild auth and settings on shared sdkwork ui"
```

## Task 4: Rebuild overview, users, tenants, and coupons on shared workbench primitives

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-overview\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-users\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-tenants\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-coupons\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-crud-ux.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-product-experience.test.mjs`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-crud-ux.test.mjs`

- [ ] **Step 1: Write the failing page-pattern tests**

```javascript
test('standard admin pages use CrudWorkbench or shared cards without adminx layout classes', () => {
  const users = read('packages/sdkwork-router-admin-users/src/index.tsx');

  assert.match(users, /CrudWorkbench|ManagementWorkbench/);
  assert.doesNotMatch(users, /adminx-/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/admin-crud-ux.test.mjs tests/admin-product-experience.test.mjs`
Expected: FAIL because the standard CRUD pages still use legacy wrappers and classes

- [ ] **Step 3: Write minimal implementation**

```tsx
<CrudWorkbench
  title="Users"
  filters={<Toolbar ... />}
  table={{ columns, rows, getRowKey }}
/>
```

Use shared `StatCard`, `Card`, `Badge`, `Dialog`, and shared workbench patterns. Remove all `adminx-*` layout classes from these pages.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/admin-crud-ux.test.mjs tests/admin-product-experience.test.mjs`
Expected: PASS

- [ ] **Step 5: Run type verification**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add packages/sdkwork-router-admin-overview packages/sdkwork-router-admin-users packages/sdkwork-router-admin-tenants packages/sdkwork-router-admin-coupons tests/admin-crud-ux.test.mjs tests/admin-product-experience.test.mjs
git commit -m "feat: standardize core admin pages on shared workbench ui"
```

## Task 5: Rebuild catalog and API router pages on shared management workbench patterns

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-catalog\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-apirouter\src\pages\GatewayAccessPage.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-apirouter\src\pages\GatewayRoutesPage.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-apirouter\src\pages\GatewayModelMappingsPage.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-apirouter\src\pages\GatewayUsagePage.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-product-experience.test.mjs`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-product-experience.test.mjs`

- [ ] **Step 1: Write the failing complex-workbench tests**

```javascript
test('catalog and api router pages use shared management workbench patterns', () => {
  const catalog = read('packages/sdkwork-router-admin-catalog/src/index.tsx');
  const routes = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayRoutesPage.tsx');

  assert.match(catalog, /ManagementWorkbench|CrudWorkbench/);
  assert.match(routes, /ManagementWorkbench|CrudWorkbench/);
  assert.doesNotMatch(catalog, /adminx-/);
  assert.doesNotMatch(routes, /adminx-/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/admin-product-experience.test.mjs`
Expected: FAIL because these pages still depend on old local UI

- [ ] **Step 3: Write minimal implementation**

```tsx
<ManagementWorkbench
  title="Catalog"
  filters={<Toolbar ... />}
  main={{ title: 'Catalog workbench', children: <DataTable ... /> }}
  detail={{ title: 'Selection details', children: <DescriptionList ... /> }}
/>
```

Use shared tables, badges, dialogs, drawers, and detail panels. Remove all `adminx-*` classes.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/admin-product-experience.test.mjs`
Expected: PASS

- [ ] **Step 5: Run type verification**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add packages/sdkwork-router-admin-catalog packages/sdkwork-router-admin-apirouter tests/admin-product-experience.test.mjs
git commit -m "feat: rebuild admin catalog and gateway workbenches"
```

## Task 6: Rebuild traffic and operations for dense-data shared UI

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-traffic\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-operations\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-product-experience.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-table-polish.test.mjs`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-table-polish.test.mjs`

- [ ] **Step 1: Write the failing dense-data tests**

```javascript
test('traffic and operations rely on shared table and filter patterns without adminx classes', () => {
  const traffic = read('packages/sdkwork-router-admin-traffic/src/index.tsx');
  const operations = read('packages/sdkwork-router-admin-operations/src/index.tsx');

  assert.match(traffic, /ManagementWorkbench/);
  assert.match(operations, /ManagementWorkbench/);
  assert.doesNotMatch(traffic, /adminx-/);
  assert.doesNotMatch(operations, /adminx-/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/admin-table-polish.test.mjs tests/admin-product-experience.test.mjs`
Expected: FAIL because both pages still use old classes and old table compositions

- [ ] **Step 3: Write minimal implementation**

```tsx
<ManagementWorkbench
  title="Traffic"
  filters={<Toolbar ... />}
  main={{ title: 'Usage records', children: <DataTable ... /> }}
/>
```

Use shared filter bars, badges, tables, stats, and feedback states. Remove all legacy classes.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/admin-table-polish.test.mjs tests/admin-product-experience.test.mjs`
Expected: PASS

- [ ] **Step 5: Run type verification**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add packages/sdkwork-router-admin-traffic packages/sdkwork-router-admin-operations tests/admin-table-polish.test.mjs tests/admin-product-experience.test.mjs
git commit -m "feat: standardize dense data admin pages on shared ui"
```

## Task 7: Remove the legacy commons package and clean package dependencies

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\*\package.json`
- Delete: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-commons\src\index.tsx`
- Delete: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\packages\sdkwork-router-admin-commons\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tsconfig.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-architecture.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-claw-foundation-parity.test.mjs`

- [ ] **Step 1: Write the failing removal tests**

```javascript
test('feature packages no longer depend on sdkwork-router-admin-commons', () => {
  const matches = rg('sdkwork-router-admin-commons', 'packages');
  assert.equal(matches.length, 0);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/admin-architecture.test.mjs tests/admin-claw-foundation-parity.test.mjs`
Expected: FAIL because the old package and imports still exist

- [ ] **Step 3: Write minimal implementation**

```json
{
  "dependencies": {
    "@sdkwork/ui-pc-react": "file:../../../sdkwork-ui/sdkwork-ui-pc-react"
  }
}
```

Update every consumer package to import directly from `@sdkwork/ui-pc-react` or feature-local helpers, then delete the old package files and path mappings.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/admin-architecture.test.mjs tests/admin-claw-foundation-parity.test.mjs`
Expected: PASS

- [ ] **Step 5: Run dependency and type verification**

Run: `pnpm install`
Expected: PASS with the updated dependency graph

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add package.json tsconfig.json packages tests
git commit -m "refactor: remove legacy router admin ui layer"
```

## Task 8: Final verification and documentation refresh

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\README.md`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-ui-framework-adoption.test.mjs`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-page-patterns.test.mjs`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-product-experience.test.mjs`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-shell-parity.test.mjs`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-crud-ux.test.mjs`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-table-polish.test.mjs`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-overview-runtime.test.mjs`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-i18n-coverage.test.mjs`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-admin\tests\admin-desktop-api-base.test.mjs`

- [ ] **Step 1: Update documentation to the new architecture**

```md
- shared UI framework: `@sdkwork/ui-pc-react`
- shell and feature pages built from shared workbench and settings patterns
- no local `adminx-*` design system remains
```

- [ ] **Step 2: Run the focused architecture and experience tests**

Run: `node --test tests/admin-ui-framework-adoption.test.mjs tests/admin-page-patterns.test.mjs tests/admin-product-experience.test.mjs tests/admin-shell-parity.test.mjs tests/admin-crud-ux.test.mjs tests/admin-table-polish.test.mjs`
Expected: PASS

- [ ] **Step 3: Run the broader app verification**

Run: `node --test tests/admin-overview-runtime.test.mjs tests/admin-i18n-coverage.test.mjs tests/admin-desktop-api-base.test.mjs`
Expected: PASS

- [ ] **Step 4: Run the final workspace checks**

Run: `pnpm typecheck`
Expected: PASS

Run: `pnpm build`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add README.md tests
git commit -m "docs: finalize router admin shared ui migration"
```
