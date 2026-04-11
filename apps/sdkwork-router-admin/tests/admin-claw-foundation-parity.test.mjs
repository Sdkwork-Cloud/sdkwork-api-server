import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');
const clawRoot = path.resolve(appRoot, '..', '..', '..', 'claw-studio');

function readFromApp(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function readFromClaw(relativePath) {
  return readFileSync(path.join(clawRoot, relativePath), 'utf8');
}

function readFirstExistingClaw(candidates) {
  for (const relativePath of candidates) {
    const absolutePath = path.join(clawRoot, relativePath);
    if (existsSync(absolutePath)) {
      return readFileSync(absolutePath, 'utf8');
    }
  }

  throw new Error(`Unable to resolve claw reference from candidates: ${candidates.join(', ')}`);
}

test('admin root imports shared ui css while the shell package owns layout host primitives', () => {
  const main = readFromApp('src/main.tsx');
  const themeCss = readFromApp('src/theme.css');
  const shellEntry = readFromApp('packages/sdkwork-router-admin-shell/src/index.ts');
  const shellHost = readFromApp('packages/sdkwork-router-admin-shell/src/styles/shell-host.css');

  assert.match(main, /@sdkwork\/ui-pc-react\/styles\.css/);
  assert.match(main, /\.\/theme\.css/);
  assert.match(themeCss, /@source "\.\/";/);
  assert.match(themeCss, /@source "\.\.\/packages";/);
  assert.match(themeCss, /--admin-shell-background/);
  assert.match(themeCss, /--admin-sidebar-background/);
  assert.match(themeCss, /--admin-content-background/);
  assert.match(themeCss, /--admin-sidebar-text:/);
  assert.match(themeCss, /--admin-sidebar-item-hover:/);
  assert.match(themeCss, /--admin-sidebar-popover-background:/);
  assert.match(themeCss, /--admin-sidebar-edge-background:/);
  assert.match(shellEntry, /\.\/styles\/shell-host\.css/);
  assert.match(shellHost, /admin-shell-host/);
  assert.match(shellHost, /admin-shell-route-scroll/);
  assert.match(shellHost, /admin-shell-sidebar-resize-handle/);
  assert.match(shellHost, /data-sdk-shell='router-admin-desktop'/);
  assert.doesNotMatch(shellHost, /admin-shell-auth-stage/);
});

test('admin no longer ships the legacy commons package and keeps localization in core', () => {
  const coreIndex = readFromApp('packages/sdkwork-router-admin-core/src/index.tsx');
  const coreI18n = readFromApp('packages/sdkwork-router-admin-core/src/i18n.tsx');

  assert.match(coreIndex, /AdminI18nProvider/);
  assert.match(coreIndex, /useAdminI18n/);
  assert.match(coreI18n, /translateAdminText/);
  assert.match(coreI18n, /ADMIN_LOCALE_OPTIONS/);
  assert.equal(
    existsSync(path.join(appRoot, 'packages', 'sdkwork-router-admin-commons', 'package.json')),
    false,
  );
});

test('admin modal forms reuse shared Textarea instead of raw textarea tags', () => {
  const files = [
    'packages/sdkwork-router-admin-tenants/src/page/ApiKeyDialog.tsx',
    'packages/sdkwork-router-admin-catalog/src/page/CatalogCredentialDialog.tsx',
    'packages/sdkwork-router-admin-catalog/src/page/CatalogChannelModelDialog.tsx',
    'packages/sdkwork-router-admin-apirouter/src/pages/mappings/GatewayModelMappingEditorDialog.tsx',
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayApiKeyCreateDialog.tsx',
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayApiKeyEditDialog.tsx',
  ];

  assert.equal(
    existsSync(
      path.join(
        appRoot,
        'packages',
        'sdkwork-router-admin-coupons',
        'src',
        'page',
        'CouponDialog.tsx',
      ),
    ),
    false,
  );

  for (const relativePath of files) {
    const source = readFromApp(relativePath);

    assert.match(source, /Textarea/);
    assert.doesNotMatch(source, /<textarea/);
  }
});

test('admin shell chrome keeps shared desktop shell while sidebar interaction styling mirrors claw-studio', () => {
  const layout = readFromApp('packages/sdkwork-router-admin-shell/src/application/layouts/MainLayout.tsx');
  const header = readFromApp('packages/sdkwork-router-admin-shell/src/components/AppHeader.tsx');
  const sidebar = readFromApp('packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx');
  const routePrefetch = readFromApp(
    'packages/sdkwork-router-admin-shell/src/application/router/routePrefetch.ts',
  );

  assert.match(layout, /relative flex h-screen flex-col overflow-hidden/);
  assert.match(layout, /<Sidebar \/>/);
  assert.match(layout, /<AppHeader \/>/);
  assert.match(layout, /admin-shell-content/);
  assert.match(layout, /\[background:var\(--admin-shell-background\)\]/);
  assert.match(layout, /bg-\[var\(--admin-content-background\)\]/);
  assert.doesNotMatch(layout, /DesktopShellFrame/);
  assert.doesNotMatch(layout, /brandMark/);
  assert.doesNotMatch(layout, /SDKWork Router Admin/);
  assert.doesNotMatch(layout, /Control plane/);
  assert.match(header, /\[background:var\(--admin-header-background\)\]/);
  assert.match(header, /ShellStatus/);
  assert.match(header, /HeaderActionButton/);
  assert.match(header, /data-slot="app-header-leading"/);
  assert.match(header, /data-slot="app-header-brand"/);
  assert.match(header, /data-slot="app-header-trailing"/);
  assert.match(header, /dataSlot="app-header-search"/);
  assert.match(header, /dataSlot="app-header-refresh"/);
  assert.match(header, /t\('Router Admin'\)/);
  assert.match(header, /ROUTE_PATHS\.OVERVIEW/);
  assert.match(header, /32x32\.png/);
  assert.match(header, /import\.meta\.url/);
  assert.match(header, /Ctrl K/);
  assert.doesNotMatch(header, /Toolbar/);
  assert.doesNotMatch(header, /ToolbarGroup/);
  assert.doesNotMatch(header, /@sdkwork\/ui-pc-react\/components\/ui/);
  assert.match(sidebar, /motion\/react/);
  assert.match(sidebar, /sidebar-edge-control/);
  assert.match(sidebar, /PanelLeftOpen/);
  assert.match(sidebar, /ChevronUp/);
  assert.match(sidebar, /\[background:var\(--admin-sidebar-background\)\]/);
  assert.match(sidebar, /text-\[var\(--admin-sidebar-text\)\]/);
  assert.match(sidebar, /text-\[var\(--admin-sidebar-text-muted\)\]/);
  assert.match(sidebar, /bg-\[var\(--admin-sidebar-item-hover\)\]/);
  assert.match(sidebar, /bg-\[var\(--admin-sidebar-popover-background\)\]/);
  assert.match(sidebar, /bg-\[var\(--admin-sidebar-edge-background\)\]/);
  assert.match(sidebar, /bg-primary-500/);
  assert.match(sidebar, /text-primary-400/);
  assert.match(sidebar, /bg-primary-500\/15/);
  assert.match(sidebar, /currentSidebarWidth = isSidebarCollapsed \? COLLAPSED_SIDEBAR_WIDTH : resolvedSidebarWidth/);
  assert.match(sidebar, /data-slot="sidebar-resize-handle"/);
  assert.match(sidebar, /sidebar-user-control/);
  assert.match(sidebar, /prefetchSidebarRoute/);
  assert.match(sidebar, /scheduleSidebarRoutePrefetch/);
  assert.match(sidebar, /cancelSidebarRoutePrefetch/);
  assert.match(sidebar, /onPointerDown=\{\(\) => prefetchSidebarRoute\(item\.to\)\}/);
  assert.match(sidebar, /onMouseEnter=\{\(\) => scheduleSidebarRoutePrefetch\(item\.to\)\}/);
  assert.match(sidebar, /onMouseLeave=\{\(\) => cancelSidebarRoutePrefetch\(item\.to\)\}/);
  assert.match(sidebar, /onFocus=\{\(\) => scheduleSidebarRoutePrefetch\(item\.to\)\}/);
  assert.match(sidebar, /onBlur=\{\(\) => cancelSidebarRoutePrefetch\(item\.to\)\}/);
  assert.match(sidebar, /prefetchSidebarRoute\(accountSettingsTarget\)/);
  assert.match(routePrefetch, /createSidebarRoutePrefetchController/);
  assert.match(routePrefetch, /scheduleDelayMs = 120/);
  assert.match(routePrefetch, /sdkwork-router-admin-overview/);
  assert.match(routePrefetch, /sdkwork-router-admin-users/);
  assert.match(routePrefetch, /sdkwork-router-admin-settings/);
  assert.doesNotMatch(sidebar, /NavigationRail/);
  assert.doesNotMatch(sidebar, /DropdownMenu/);
  assert.doesNotMatch(sidebar, /AdminShellBrandMark/);
  assert.doesNotMatch(sidebar, /AvatarFallback|<Avatar|import\s*\{\s*Avatar/);
  assert.doesNotMatch(sidebar, /text-zinc-/);
  assert.doesNotMatch(sidebar, /bg-zinc-/);
  assert.doesNotMatch(sidebar, /dark:bg-zinc-/);
  assert.doesNotMatch(sidebar, /border-white\//);
  assert.doesNotMatch(sidebar, /bg-white\/\[/);
  assert.doesNotMatch(sidebar, /SDKWork Router Admin/);
  assert.doesNotMatch(sidebar, /t\('Control plane'\)/);
});

test('admin sidebar collapse heuristics and persisted preference mirror claw-studio', () => {
  const adminStore = readFromApp('packages/sdkwork-router-admin-core/src/store.ts');
  const adminAutoCollapse = readFromApp('packages/sdkwork-router-admin-core/src/sidebarAutoCollapse.ts');
  const clawStore = readFirstExistingClaw([
    'packages/sdkwork-claw-core/src/stores/useAppStore.ts',
    'packages/sdkwork-claw-core/src/store/useAppStore.ts',
  ]);

  const clawBaselineStoreSnippets = [
    'isSidebarCollapsed',
    'sidebarWidth',
    'toggleSidebar',
    'setSidebarCollapsed',
    'setSidebarWidth',
  ];

  for (const snippet of clawBaselineStoreSnippets) {
    assert.match(clawStore, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
    assert.match(adminStore, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }

  const adminAutoCollapseSnippets = [
    'const COMPACT_VIEWPORT_WIDTH = 1440;',
    'const ROOMY_VIEWPORT_WIDTH = 1600;',
    'const TIGHT_VIEWPORT_HEIGHT = 900;',
    'const HIGH_SCALE_FACTOR = 1.25;',
    'const TIGHT_EFFECTIVE_SCREEN_HEIGHT = 920;',
    'export function shouldAutoCollapseSidebar',
    'export function resolveAutoSidebarCollapsed',
  ];

  for (const snippet of adminAutoCollapseSnippets) {
    assert.match(adminAutoCollapse, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }

  const adminStoreSnippets = [
    'sidebarCollapsePreference',
    "sidebarCollapsePreference: 'auto'",
    'resolveAutoSidebarCollapsed()',
    "sidebarCollapsePreference: 'user'",
    "sidebarCollapsePreference === 'auto'",
  ];

  for (const snippet of adminStoreSnippets) {
    assert.match(adminStore, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }
});
