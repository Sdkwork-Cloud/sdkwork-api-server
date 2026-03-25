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

  assert.match(header, /data-slot="app-header-search"/);
  assert.match(header, /Search/);
  assert.match(header, /adminx-shell-header-search-shortcut/);
  assert.doesNotMatch(header, /adminx-shell-header-center/);
  assert.doesNotMatch(header, /adminx-shell-header-workspace/);
  assert.doesNotMatch(header, /data-slot="app-header-center"/);
  assert.doesNotMatch(header, /data-slot="app-header-trailing"/);
  assert.match(sidebar, /adminx-shell-sidebar-link-rail/);
  assert.match(sidebar, /adminx-shell-sidebar-surface/);
  assert.doesNotMatch(sidebar, /adminx-shell-sidebar-link-badge/);
  assert.doesNotMatch(sidebar, /adminx-shell-sidebar-user-summary/);
  assert.match(layout, /adminx-shell-atmosphere/);
  assert.match(routes, /PageFrame/);
  assert.match(routes, /adminx-page-frame/);
  assert.match(routes, /adminx-page-frame-shell/);
  assert.match(routes, /adminx-page-frame-scroll/);
  assert.match(theme, /adminx-shell-atmosphere/);
  assert.match(theme, /adminx-page-frame/);
  assert.match(theme, /adminx-page-frame-shell/);
  assert.match(theme, /adminx-page-frame-scroll/);
});

test('admin shell desktop header keeps only the left brand/search area without right-side window controls', () => {
  const header = read('packages/sdkwork-router-admin-shell/src/components/AppHeader.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');
  const tauriConfig = read('src-tauri/tauri.conf.json');

  assert.match(header, /data-tauri-drag-region/);
  assert.doesNotMatch(header, /DesktopWindowControls/);
  assert.doesNotMatch(header, /minimizeWindow|toggleMaximizeWindow|maximizeWindow/);
  assert.doesNotMatch(header, /closeWindow/);
  assert.doesNotMatch(header, /ShellStatus/);
  assert.doesNotMatch(header, /Refresh workspace/);
  assert.match(tauriConfig, /"decorations":\s*false/);
  assert.match(theme, /adminx-shell-header-main/);
  assert.doesNotMatch(theme, /\.adminx-shell-header-center\b/);
  assert.doesNotMatch(theme, /\.adminx-shell-header-center-panel\b/);
  assert.doesNotMatch(theme, /\.adminx-shell-header-workspace\b/);
  assert.doesNotMatch(theme, /\.adminx-shell-header-workspace-pill\b/);
  assert.doesNotMatch(theme, /\.adminx-shell-header-actions\b/);
  assert.doesNotMatch(theme, /\.adminx-window-controls\b/);
  assert.match(theme, /@source "\.\.\/\.\.\/\.\.\/\.\.\//);
});

test('admin header brand mark uses the same claw glyph family as the sidebar and claw-studio shell', () => {
  const header = read('packages/sdkwork-router-admin-shell/src/components/AppHeader.tsx');

  assert.match(header, /M12 2v2/);
  assert.match(header, /M15 12a3 3 0 1 1-6 0/);
  assert.doesNotMatch(header, /M5 12h14/);
});

test('admin shell sidebar footer anchors user identity and bottom settings affordance', () => {
  const sidebar = read('packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(sidebar, /useAdminWorkbench/);
  assert.match(sidebar, /data-slot="sidebar-footer-settings"/);
  assert.match(sidebar, /data-slot="sidebar-user-control"/);
  assert.match(sidebar, /adminx-shell-sidebar-avatar/);
  assert.doesNotMatch(sidebar, /adminx-shell-sidebar-secondary-copy/);
  assert.doesNotMatch(sidebar, /adminx-shell-sidebar-link-badge/);
  assert.match(theme, /adminx-shell-sidebar-footer-action/);
  assert.match(theme, /adminx-shell-sidebar-avatar/);
  assert.match(theme, /adminx-shell-sidebar-footer-user/);
});

test('admin sidebar keeps claw-style top brand rhythm and drops custom resize affordances', () => {
  const sidebar = read('packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx');

  assert.match(sidebar, /motion\.(div|aside)/);
  assert.match(sidebar, /SDKWork Router Admin/);
  assert.match(sidebar, /Control plane/);
  assert.match(sidebar, /M12 2v2/);
  assert.match(sidebar, /M15 12a3 3 0 1 1-6 0/);
  assert.match(sidebar, /w-full items-center overflow-hidden whitespace-nowrap/);
  assert.doesNotMatch(sidebar, /sidebar-resize-handle/);
  assert.doesNotMatch(sidebar, /col-resize/);
});

test('admin shell sidebar keeps a compact claw-style footer while retaining user menu controls', () => {
  const sidebar = read('packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(sidebar, /isUserMenuOpen/);
  assert.match(sidebar, /data-slot="sidebar-user-control"/);
  assert.match(sidebar, /adminx-shell-sidebar-user-menu/);
  assert.match(sidebar, /layoutId="adminx-sidebar-active-indicator"/);
  assert.match(theme, /adminx-shell-sidebar-footer-action/);
  assert.match(theme, /adminx-shell-sidebar-user-control/);
  assert.match(theme, /adminx-shell-sidebar-user-menu/);
  assert.doesNotMatch(theme, /adminx-shell-sidebar-secondary-action/);
});

test('admin sidebar route items use direct claw-style composition and keep an always-available expand affordance', () => {
  const sidebar = read('packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx');

  assert.match(sidebar, /mx-auto h-10 w-10 justify-center/);
  assert.match(sidebar, /justify-between px-3 py-2\.5/);
  assert.match(sidebar, /hover:bg-zinc-800\/50 hover:text-zinc-200/);
  assert.match(sidebar, /text-\[14px\] tracking-tight/);
  assert.match(sidebar, /onClick=\{isSidebarCollapsed \? toggleSidebar : undefined\}/);
  assert.match(sidebar, /title=\{isSidebarCollapsed \? t\('Expand sidebar'\) : undefined\}/);
  assert.match(sidebar, /aria-label=\{isSidebarCollapsed \? t\('Expand sidebar'\) : undefined\}/);
  assert.match(sidebar, /title=\{t\('Collapse sidebar'\)\}/);
  assert.doesNotMatch(sidebar, /adminx-shell-sidebar-link-leading/);
  assert.doesNotMatch(sidebar, /adminx-shell-sidebar-link-copy/);
  assert.doesNotMatch(sidebar, /adminx-shell-sidebar-footer-leading/);
  assert.doesNotMatch(sidebar, /adminx-shell-sidebar-footer-label/);
});

test('admin sidebar pins settings and user controls to the rail bottom and clamps the user menu to the viewport', () => {
  const sidebar = read('packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(sidebar, /const userControlRef = useRef/);
  assert.match(sidebar, /const userMenuPanelRef = useRef/);
  assert.match(sidebar, /const \[userMenuStyle, setUserMenuStyle\] = useState/);
  assert.match(sidebar, /getBoundingClientRect/);
  assert.match(sidebar, /window\.innerWidth/);
  assert.match(sidebar, /window\.innerHeight/);
  assert.match(sidebar, /style=\{userMenuStyle\}/);
  assert.match(theme, /\.adminx-shell-sidebar-footer\s*\{[^}]*margin-top:\s*auto;/s);
  assert.match(theme, /\.adminx-shell-sidebar-footer\s*\{[^}]*flex-shrink:\s*0;/s);
  assert.match(theme, /\.adminx-shell-sidebar-user-menu\s*\{[^}]*position:\s*fixed;/s);
  assert.doesNotMatch(theme, /\.adminx-shell-sidebar-user-menu\s*\{[^}]*bottom:\s*calc\(100% \+ 10px\)/s);
  assert.doesNotMatch(theme, /\.adminx-shell-sidebar-user-menu\.is-collapsed\s*\{[^}]*left:\s*calc\(100% \+ 10px\)/s);
});

test('admin shell keeps sidebar and main pinned to the viewport-height workspace instead of letting content stretch the rail', () => {
  const sidebar = read('packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(sidebar, /adminx-shell-sidebar relative z-20 flex min-h-0 shrink-0 self-stretch/);
  assert.match(sidebar, /adminx-shell-sidebar-inner adminx-shell-sidebar-surface flex min-h-0 w-full flex-col/);
  assert.match(sidebar, /scrollbar-hide mt-4 min-h-0 flex-1 space-y-6 overflow-x-hidden overflow-y-auto/);
  assert.match(theme, /\.adminx-shell\s*\{[^}]*height:\s*100vh;[^}]*height:\s*100dvh;/s);
  assert.match(theme, /\.adminx-shell-sidebar\s*\{[^}]*min-height:\s*0;[^}]*align-self:\s*stretch;/s);
  assert.match(theme, /\.adminx-shell-sidebar-inner\s*\{[^}]*min-height:\s*0;/s);
  assert.match(theme, /\.adminx-shell-main\s*\{[^}]*min-height:\s*0;/s);
});

test('admin settings center keeps claw-style left-nav composition and query-param tabs', () => {
  const settings = read('packages/sdkwork-router-admin-settings/src/Settings.tsx');
  const packageJson = read('packages/sdkwork-router-admin-settings/package.json');
  const appearance = read('packages/sdkwork-router-admin-settings/src/AppearanceSettings.tsx');
  const navigation = read('packages/sdkwork-router-admin-settings/src/NavigationSettings.tsx');

  assert.match(settings, /useSearchParams/);
  assert.match(settings, /motion\.(div|section)/);
  assert.match(settings, /from 'motion\/react'/);
  assert.match(packageJson, /"motion"/);
  assert.match(settings, /data-settings-tab/);
  assert.match(settings, /Appearance/);
  assert.match(settings, /Navigation/);
  assert.match(settings, /Workspace/);
  assert.match(settings, /flex h-full bg-zinc-50\/50 dark:bg-zinc-950\/50/);
  assert.match(
    settings,
    /flex w-72 shrink-0 flex-col border-r border-zinc-200 bg-zinc-50\/80 backdrop-blur-xl dark:border-zinc-800 dark:bg-zinc-900\/80/,
  );
  assert.match(settings, /scrollbar-hide flex-1 overflow-x-hidden overflow-y-auto/);
  assert.match(settings, /mx-auto w-full max-w-5xl p-8 md:p-12/);
  assert.match(appearance, /theme mode/i);
  assert.match(navigation, /sidebar visibility/i);
});

test('admin settings center removes custom summary and stage wrappers so the shell matches claw-studio directly', () => {
  const settings = read('packages/sdkwork-router-admin-settings/src/Settings.tsx');
  const appearance = read('packages/sdkwork-router-admin-settings/src/AppearanceSettings.tsx');
  const navigation = read('packages/sdkwork-router-admin-settings/src/NavigationSettings.tsx');
  const workspace = read('packages/sdkwork-router-admin-settings/src/WorkspaceSettings.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.doesNotMatch(settings, /admin-shell-settings-nav-summary/);
  assert.doesNotMatch(settings, /admin-shell-settings-panel-stage/);
  assert.doesNotMatch(settings, /admin-shell-settings-stage-metrics/);
  assert.doesNotMatch(settings, /admin-shell-settings-nav-shell-preview/);
  assert.doesNotMatch(appearance, /theme signature|shell visual snapshot/i);
  assert.doesNotMatch(navigation, /sidebar preview|live rail/i);
  assert.match(workspace, /persistence|canvas/i);
  assert.doesNotMatch(theme, /admin-shell-settings-nav-summary/);
  assert.doesNotMatch(theme, /admin-shell-settings-panel-stage/);
  assert.doesNotMatch(theme, /admin-shell-settings-stage-metrics/);
  assert.doesNotMatch(theme, /admin-shell-settings-nav-shell-preview/);
});

test('admin settings panels use claw-style sections instead of admin-only KPI summary cards', () => {
  const shared = read('packages/sdkwork-router-admin-settings/src/Shared.tsx');
  const general = read('packages/sdkwork-router-admin-settings/src/GeneralSettings.tsx');
  const appearance = read('packages/sdkwork-router-admin-settings/src/AppearanceSettings.tsx');
  const navigation = read('packages/sdkwork-router-admin-settings/src/NavigationSettings.tsx');
  const workspace = read('packages/sdkwork-router-admin-settings/src/WorkspaceSettings.tsx');

  assert.match(shared, /overflow-hidden rounded-\[1\.5rem\] border border-zinc-200\/80 bg-white shadow-sm/);
  assert.match(shared, /text-\[15px\] font-bold tracking-tight text-zinc-900/);
  assert.doesNotMatch(general, /admin-shell-settings-kpi/);
  assert.doesNotMatch(appearance, /admin-shell-settings-kpi/);
  assert.doesNotMatch(navigation, /admin-shell-settings-kpi/);
  assert.doesNotMatch(workspace, /admin-shell-settings-kpi/);
  assert.doesNotMatch(general, /admin-shell-settings-card-head/);
  assert.doesNotMatch(appearance, /admin-shell-settings-card-head/);
  assert.doesNotMatch(navigation, /admin-shell-settings-card-head/);
  assert.doesNotMatch(workspace, /admin-shell-settings-card-head/);
  assert.doesNotMatch(shared, /inline-flex h-9 w-9 items-center justify-center rounded-xl/);
});

test('theme manager applies claw-compatible root classes for shared dark styling', () => {
  const themeManager = read(
    'packages/sdkwork-router-admin-shell/src/application/providers/ThemeManager.tsx',
  );

  assert.match(themeManager, /classList\.toggle\('dark'/);
  assert.match(themeManager, /theme-light/);
  assert.match(themeManager, /theme-dark/);
});

test('admin shell typography stays on a claw-compatible system sans stack', () => {
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(theme, /font-family:\s*ui-sans-serif,\s*system-ui/s);
  assert.doesNotMatch(theme, /Space Grotesk/);
  assert.doesNotMatch(theme, /Avenir Next/);
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

test('admin commons exports claw-aligned shadcn primitives for future page parity', () => {
  const commons = read('packages/sdkwork-router-admin-commons/src/index.tsx');
  const settings = read('packages/sdkwork-router-admin-settings/src/Settings.tsx');

  assert.match(commons, /@radix-ui\/react-dialog/);
  assert.match(commons, /@radix-ui\/react-label/);
  assert.match(commons, /@radix-ui\/react-slot/);
  assert.match(commons, /class-variance-authority/);
  assert.match(commons, /tailwind-merge/);
  assert.match(commons, /export function cn/);
  assert.match(commons, /export (const|function) Button/);
  assert.match(commons, /export const Input/);
  assert.match(commons, /export const Label/);
  assert.match(commons, /export const Dialog =/);
  assert.match(commons, /export const DialogContent/);
  assert.match(settings, /Input/);
});
