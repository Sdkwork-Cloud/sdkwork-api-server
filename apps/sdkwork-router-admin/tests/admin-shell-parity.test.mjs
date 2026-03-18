import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('admin shell header, sidebar, and page frame mirror claw-style shell slots', () => {
  const header = read('packages/sdkwork-router-admin-shell/src/components/AppHeader.tsx');
  const sidebar = read('packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx');
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(header, /adminx-shell-header-center/);
  assert.match(header, /adminx-shell-header-workspace/);
  assert.match(sidebar, /adminx-shell-sidebar-link-rail/);
  assert.match(sidebar, /adminx-shell-sidebar-link-badge/);
  assert.match(routes, /PageFrame/);
  assert.match(routes, /adminx-page-frame/);
  assert.match(theme, /adminx-shell-header-center/);
  assert.match(theme, /adminx-page-frame/);
});

test('admin shell desktop header supports tauri drag regions and native window controls', () => {
  const header = read('packages/sdkwork-router-admin-shell/src/components/AppHeader.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');
  const tauriConfig = read('src-tauri/tauri.conf.json');

  assert.match(header, /DesktopWindowControls/);
  assert.match(header, /data-tauri-drag-region/);
  assert.match(header, /minimizeWindow|toggleMaximizeWindow|maximizeWindow/);
  assert.match(header, /closeWindow/);
  assert.match(theme, /adminx-window-controls/);
  assert.match(tauriConfig, /"decorations":\s*false/);
});

test('admin shell sidebar footer anchors user identity and bottom settings affordance', () => {
  const sidebar = read('packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(sidebar, /useAdminWorkbench/);
  assert.match(sidebar, /adminx-shell-sidebar-profile/);
  assert.match(sidebar, /adminx-shell-sidebar-avatar/);
  assert.match(sidebar, /adminx-shell-sidebar-footer-settings/);
  assert.match(theme, /adminx-shell-sidebar-profile/);
  assert.match(theme, /adminx-shell-sidebar-avatar/);
  assert.match(theme, /adminx-shell-sidebar-footer-settings/);
});

test('admin shell sidebar mirrors claw account-control richness with popover state and animated active affordances', () => {
  const sidebar = read('packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(sidebar, /isUserMenuOpen/);
  assert.match(sidebar, /data-slot="sidebar-user-control"/);
  assert.match(sidebar, /adminx-shell-sidebar-user-menu/);
  assert.match(sidebar, /layoutId="adminx-sidebar-active-indicator"/);
  assert.match(theme, /adminx-shell-sidebar-user-control/);
  assert.match(theme, /adminx-shell-sidebar-user-menu/);
  assert.match(theme, /adminx-shell-sidebar-secondary-action/);
});

test('admin settings center keeps claw-style left-nav composition and query-param tabs', () => {
  const settings = read('packages/sdkwork-router-admin-settings/src/Settings.tsx');
  const appearance = read('packages/sdkwork-router-admin-settings/src/AppearanceSettings.tsx');
  const navigation = read('packages/sdkwork-router-admin-settings/src/NavigationSettings.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(settings, /useSearchParams/);
  assert.match(settings, /data-settings-tab/);
  assert.match(settings, /Appearance/);
  assert.match(settings, /Navigation/);
  assert.match(settings, /Workspace/);
  assert.match(appearance, /theme mode/i);
  assert.match(navigation, /sidebar preview/i);
  assert.match(theme, /admin-shell-settings-tab-icon/);
  assert.match(theme, /admin-shell-settings-preview/);
});

test('admin settings center exposes claw-style nav summary and a dedicated detail stage', () => {
  const settings = read('packages/sdkwork-router-admin-settings/src/Settings.tsx');
  const appearance = read('packages/sdkwork-router-admin-settings/src/AppearanceSettings.tsx');
  const navigation = read('packages/sdkwork-router-admin-settings/src/NavigationSettings.tsx');
  const workspace = read('packages/sdkwork-router-admin-settings/src/WorkspaceSettings.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(settings, /admin-shell-settings-nav-summary/);
  assert.match(settings, /admin-shell-settings-panel-stage/);
  assert.match(appearance, /theme signature|live theme/i);
  assert.match(navigation, /visible routes|live rail/i);
  assert.match(workspace, /persistence|canvas/i);
  assert.match(theme, /admin-shell-settings-nav-summary/);
  assert.match(theme, /admin-shell-settings-panel-stage/);
});

test('theme manager applies claw-compatible root classes for shared dark styling', () => {
  const themeManager = read(
    'packages/sdkwork-router-admin-shell/src/application/providers/ThemeManager.tsx',
  );

  assert.match(themeManager, /classList\.toggle\('dark'/);
  assert.match(themeManager, /theme-light/);
  assert.match(themeManager, /theme-dark/);
});

test('content primitives expose theme-aware rows, placeholders, and surface states', () => {
  const commons = read('packages/sdkwork-router-admin-commons/src/index.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(commons, /adminx-table-row/);
  assert.match(commons, /Dialog|DialogContent|DialogTrigger/);
  assert.match(theme, /adminx-table-row:hover/);
  assert.match(theme, /::placeholder/);
  assert.match(theme, /option\s*\{/);
});
