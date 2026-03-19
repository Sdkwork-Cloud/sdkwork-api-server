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
  const layout = read('packages/sdkwork-router-admin-shell/src/application/layouts/MainLayout.tsx');
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(header, /adminx-shell-header-center/);
  assert.match(header, /adminx-shell-header-workspace/);
  assert.match(header, /data-slot="app-header-search"/);
  assert.match(header, /Search/);
  assert.match(header, /adminx-shell-header-search-shortcut/);
  assert.match(sidebar, /adminx-shell-sidebar-link-rail/);
  assert.match(sidebar, /adminx-shell-sidebar-link-badge/);
  assert.match(sidebar, /adminx-shell-sidebar-surface/);
  assert.match(sidebar, /adminx-shell-sidebar-user-summary/);
  assert.match(layout, /adminx-shell-atmosphere/);
  assert.match(routes, /PageFrame/);
  assert.match(routes, /adminx-page-frame/);
  assert.match(routes, /adminx-page-frame-shell/);
  assert.match(routes, /adminx-page-frame-scroll/);
  assert.match(theme, /adminx-shell-atmosphere/);
  assert.match(theme, /adminx-shell-header-center/);
  assert.match(theme, /adminx-page-frame/);
  assert.match(theme, /adminx-page-frame-shell/);
  assert.match(theme, /adminx-page-frame-scroll/);
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
  assert.match(settings, /admin-shell-settings-nav-shell-preview/);
  assert.match(theme, /admin-shell-settings-nav-shell-preview/);
});

test('admin settings center exposes claw-style nav summary and a dedicated detail stage', () => {
  const settings = read('packages/sdkwork-router-admin-settings/src/Settings.tsx');
  const appearance = read('packages/sdkwork-router-admin-settings/src/AppearanceSettings.tsx');
  const navigation = read('packages/sdkwork-router-admin-settings/src/NavigationSettings.tsx');
  const workspace = read('packages/sdkwork-router-admin-settings/src/WorkspaceSettings.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(settings, /admin-shell-settings-nav-summary/);
  assert.match(settings, /admin-shell-settings-panel-stage/);
  assert.match(settings, /admin-shell-settings-stage-metrics/);
  assert.match(appearance, /theme signature|live theme/i);
  assert.match(navigation, /visible routes|live rail/i);
  assert.match(workspace, /persistence|canvas/i);
  assert.match(theme, /admin-shell-settings-nav-summary/);
  assert.match(theme, /admin-shell-settings-panel-stage/);
  assert.match(theme, /admin-shell-settings-stage-metrics/);
});

test('theme manager applies claw-compatible root classes for shared dark styling', () => {
  const themeManager = read(
    'packages/sdkwork-router-admin-shell/src/application/providers/ThemeManager.tsx',
  );

  assert.match(themeManager, /classList\.toggle\('dark'/);
  assert.match(themeManager, /theme-light/);
  assert.match(themeManager, /theme-dark/);
});

test('admin auth surface mirrors claw auth route shape while keeping admin-only login truthfulness', () => {
  const auth = read('packages/sdkwork-router-admin-auth/src/index.tsx');
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');
  const routePaths = read('packages/sdkwork-router-admin-core/src/routePaths.ts');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');
  const authRouteElementBlock = routes.match(/const authRouteElement = !authResolved \? \([\s\S]*?\n  \);/);

  assert.match(routePaths, /AUTH/);
  assert.match(routePaths, /REGISTER/);
  assert.match(routePaths, /FORGOT_PASSWORD/);
  assert.match(routes, /ROUTE_PATHS\.AUTH/);
  assert.match(routes, /ROUTE_PATHS\.REGISTER/);
  assert.match(routes, /ROUTE_PATHS\.FORGOT_PASSWORD/);
  assert.ok(authRouteElementBlock);
  assert.match(authRouteElementBlock[0], /<AdminLoginPage/);
  assert.doesNotMatch(authRouteElementBlock[0], /<PageFrame/);
  assert.match(auth, /resolveAuthMode/);
  assert.match(auth, /mode === 'register'/);
  assert.match(auth, /mode === 'forgot'/);
  assert.match(auth, /QrCode|QR Login/);
  assert.match(auth, /Github/);
  assert.match(auth, /Chrome|Google/);
  assert.match(auth, /adminx-auth-frame/);
  assert.match(auth, /adminx-auth-aside/);
  assert.match(auth, /adminx-auth-content/);
  assert.match(auth, /adminx-auth-provider-grid/);
  assert.match(auth, /adminx-auth-mode-switch/);
  assert.match(auth, /Scan to Login/);
  assert.match(auth, /Use the SDKWork mobile app to scan the QR code for instant access\./);
  assert.match(auth, /Welcome back/);
  assert.match(auth, /Enter your details to access your account\./);
  assert.match(auth, /Create an account/);
  assert.match(auth, /Join us to start building amazing things\./);
  assert.match(auth, /Reset password/);
  assert.match(auth, /Enter your email to receive a reset link\./);
  assert.match(auth, /John Doe/);
  assert.match(auth, /you@example\.com/);
  assert.match(auth, /Enter your password/);
  assert.match(auth, /Send Reset Link/);
  assert.match(auth, /Don't have an account\?/);
  assert.match(auth, /Already have an account\?/);
  assert.match(auth, /Back to login/);
  assert.match(auth, /Operator account requests/);
  assert.match(auth, /Password resets are managed/);
  assert.match(theme, /adminx-auth-frame/);
  assert.match(theme, /adminx-auth-aside/);
  assert.match(theme, /adminx-auth-content/);
  assert.match(theme, /adminx-auth-provider-grid/);
  assert.match(theme, /adminx-auth-mode-switch/);
  assert.match(theme, /max-width:\s*56rem/);
  assert.match(theme, /border-radius:\s*24px/);
  assert.match(theme, /font-size:\s*24px/);
  assert.match(theme, /font-size:\s*30px/);
  assert.match(theme, /max-width:\s*200px/);
  assert.match(theme, /\.adminx-auth-input-shell input\s*\{[^}]*border-radius:\s*12px;/);
  assert.match(theme, /\.adminx-auth-input-shell input\s*\{[^}]*font-size:\s*14px;/);
  assert.match(theme, /\.adminx-auth-primary-button\s*\{[^}]*border-radius:\s*12px;/);
  assert.match(theme, /\.adminx-auth-primary-button\s*\{[^}]*font-size:\s*14px;/);
  assert.match(theme, /\.adminx-auth-provider-button\s*\{[^}]*border-radius:\s*12px;/);
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
