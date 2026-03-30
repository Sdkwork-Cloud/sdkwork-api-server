import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal shell adopts a claw-style router and shell composition', () => {
  const coreEntry = read('packages/sdkwork-router-portal-core/src/index.tsx');
  const appProviders = read('packages/sdkwork-router-portal-core/src/application/providers/AppProviders.tsx');
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');
  const layout = read('packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx');
  const header = read('packages/sdkwork-router-portal-core/src/components/AppHeader.tsx');

  assert.match(coreEntry, /PortalProductApp/);
  assert.match(appProviders, /BrowserRouter/);
  assert.match(appProviders, /basename="\s*\/portal\s*"/);
  assert.match(appRoutes, /Routes/);
  assert.match(appRoutes, /Route/);
  assert.match(layout, /<Sidebar/);
  assert.match(layout, /<AppHeader/);
  assert.match(layout, /ConfigCenter/);
  assert.doesNotMatch(layout, /ShellStatus/);
  assert.match(header, /bg-white\/72 backdrop-blur-xl dark:bg-zinc-950\/78/);
  assert.match(header, /SDKWork Router/);
  assert.doesNotMatch(header, /Portal Workspace/);
});

test('portal header behaves like a claw-style desktop titlebar without a centered workspace strip', () => {
  const layout = read('packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx');
  const header = read('packages/sdkwork-router-portal-core/src/components/AppHeader.tsx');
  const windowControls = read('packages/sdkwork-router-portal-core/src/components/WindowControls.tsx');

  assert.match(layout, /<AppHeader \/>/);
  assert.match(header, /data-tauri-drag-region/);
  assert.match(header, /WindowControls/);
  assert.match(header, /data-slot="app-header-leading"/);
  assert.match(header, /data-slot="app-header-trailing"/);
  assert.doesNotMatch(header, /data-slot="app-header-center"/);
  assert.doesNotMatch(header, /Current workspace|Workspace context/);
  assert.doesNotMatch(header, /workspace\?\.project\.name|workspace\?\.tenant\.name|storedWorkspace/);
  assert.doesNotMatch(header, /onOpenConfigCenter/);
  assert.doesNotMatch(header, /workspace:\s*['"]/);
  assert.doesNotMatch(header, /Workspace shell/);
  assert.doesNotMatch(header, /Config center/);
  assert.doesNotMatch(header, /Palette/);
  assert.doesNotMatch(header, /Settings2/);
  assert.match(windowControls, /minimizeWindow/);
  assert.match(windowControls, /maximizeWindow/);
  assert.match(windowControls, /closeWindow/);
  assert.match(windowControls, /hover:bg-zinc-950\/\[0\.06\]|hover:bg-white\/\[0\.1\]/);
  assert.match(header, /M12 2v2/);
  assert.match(header, /M15 12a3 3 0 1 1-6 0/);
  assert.doesNotMatch(header, /Sparkles/);
});

test('portal shell sidebar keeps claw-studio collapse rhythm with resize and edge affordances', () => {
  const sidebar = read('packages/sdkwork-router-portal-core/src/components/Sidebar.tsx');
  const packageJson = read('packages/sdkwork-router-portal-core/package.json');
  const store = read('packages/sdkwork-router-portal-core/src/store/usePortalShellStore.ts');
  const configCenter = read('packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx');
  const preferences = read('packages/sdkwork-router-portal-core/src/lib/portalPreferences.ts');

  assert.match(packageJson, /"motion"/);
  assert.match(store, /isSidebarCollapsed/);
  assert.match(store, /sidebarWidth/);
  assert.match(store, /sidebarCollapsePreference/);
  assert.match(store, /resolveAutoSidebarCollapsed/);
  assert.match(store, /hiddenSidebarItems/);
  assert.match(sidebar, /w-full items-center overflow-hidden whitespace-nowrap/);
  assert.match(sidebar, /SDKWork Router/);
  assert.match(sidebar, /Developer portal/);
  assert.match(sidebar, /M12 2v2/);
  assert.match(sidebar, /M15 12a3 3 0 1 1-6 0/);
  assert.match(sidebar, /toggleSidebar/);
  assert.match(sidebar, /const \[isSidebarHovered, setIsSidebarHovered\] = useState\(false\)/);
  assert.match(sidebar, /const \[isSidebarResizing, setIsSidebarResizing\] = useState\(false\)/);
  assert.match(sidebar, /showEdgeAffordances = !isSidebarCollapsed && \(isSidebarHovered \|\| isSidebarResizing\)/);
  assert.match(sidebar, /data-slot="sidebar-edge-control"/);
  assert.match(sidebar, /data-slot="sidebar-resize-handle"/);
  assert.match(sidebar, /cursor-col-resize/);
  assert.match(sidebar, /onPointerDown=\{startSidebarResize\}/);
  assert.doesNotMatch(sidebar, /Active workspace/);
  assert.match(preferences, /PORTAL_COLLAPSED_SIDEBAR_WIDTH = 72/);
  assert.match(preferences, /PORTAL_DEFAULT_SIDEBAR_WIDTH = 252/);
  assert.match(preferences, /PORTAL_MIN_SIDEBAR_WIDTH = 220/);
  assert.match(preferences, /PORTAL_MAX_SIDEBAR_WIDTH = 360/);
  assert.match(configCenter, /hiddenSidebarItems/);
  assert.match(configCenter, /themeColor/);
});

test('portal sidebar footer collapses shell actions into a dedicated profile dock', () => {
  const sidebar = read('packages/sdkwork-router-portal-core/src/components/Sidebar.tsx');
  const profileDockPath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-core',
    'src',
    'components',
    'SidebarProfileDock.tsx',
  );

  assert.equal(existsSync(profileDockPath), true);
  assert.match(sidebar, /SidebarProfileDock/);
  assert.doesNotMatch(sidebar, /Settings2/);
  assert.doesNotMatch(sidebar, /LogOut/);

  const profileDock = read('packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx');
  assert.match(profileDock, /onOpenConfigCenter/);
  assert.match(profileDock, /onLogout/);
  assert.match(profileDock, /workspace\?\.user\.display_name|workspace\?\.user\.email/);
  assert.match(profileDock, /data-slot="portal-sidebar-footer-settings"/);
  assert.match(profileDock, /data-slot="portal-sidebar-user-control"/);
  assert.doesNotMatch(profileDock, /Active workspace/);
  assert.doesNotMatch(profileDock, /Theme, sidebar, and shell preferences/);
  assert.match(profileDock, /Sign out|Logout/);
  assert.match(profileDock, /const userControlRef = useRef/);
  assert.match(profileDock, /const userMenuPanelRef = useRef/);
  assert.match(profileDock, /const \[userMenuStyle, setUserMenuStyle\] = useState/);
  assert.match(profileDock, /getBoundingClientRect/);
  assert.match(profileDock, /window\.innerWidth/);
  assert.match(profileDock, /window\.innerHeight/);
  assert.match(profileDock, /style=\{userMenuStyle\}/);
  assert.doesNotMatch(profileDock, /bottom-full left-0 right-0 mb-3/);
  assert.doesNotMatch(profileDock, /bottom-0 left-full ml-3 w-72/);
});

test('portal shell exposes a dedicated WindowControls component and desktop host scaffold', () => {
  const header = read('packages/sdkwork-router-portal-core/src/components/AppHeader.tsx');
  const packageJson = read('package.json');
  const tauriConfigPath = path.join(appRoot, 'src-tauri', 'tauri.conf.json');
  const tauriCargoPath = path.join(appRoot, 'src-tauri', 'Cargo.toml');
  const tauriIconPath = path.join(appRoot, 'src-tauri', 'icons', 'icon.ico');
  const windowControlsPath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-core',
    'src',
    'components',
    'WindowControls.tsx',
  );

  assert.equal(existsSync(windowControlsPath), true);
  assert.match(header, /WindowControls/);
  assert.doesNotMatch(header, /function DesktopWindowControls/);
  assert.equal(existsSync(tauriConfigPath), true);
  assert.equal(existsSync(tauriCargoPath), true);
  assert.equal(existsSync(tauriIconPath), true);
  assert.match(packageJson, /"tauri:dev"/);
  assert.match(packageJson, /"tauri:build"/);
  assert.match(packageJson, /run-tauri-cli\.mjs dev/);
  assert.match(packageJson, /run-tauri-cli\.mjs build/);
  assert.match(packageJson, /@tauri-apps\/cli/);

  const tauriConfig = readFileSync(tauriConfigPath, 'utf8');
  assert.match(tauriConfig, /"decorations"\s*:\s*false/);
});
