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
  assert.match(layout, /ShellStatus/);
  assert.match(header, /portal-surface-contrast|portal-glass-background/);
  assert.match(header, /SDKWork Router/);
  assert.match(header, /Portal Workspace/);
});

test('portal header behaves like a desktop titlebar with brand left and window controls right', () => {
  const header = read('packages/sdkwork-router-portal-core/src/components/AppHeader.tsx');
  const windowControls = read('packages/sdkwork-router-portal-core/src/components/WindowControls.tsx');

  assert.match(header, /data-tauri-drag-region/);
  assert.match(header, /WindowControls/);
  assert.doesNotMatch(header, /onOpenConfigCenter/);
  assert.doesNotMatch(header, /workspace:/);
  assert.doesNotMatch(header, /Workspace shell/);
  assert.doesNotMatch(header, /Config center/);
  assert.doesNotMatch(header, /Active workspace/);
  assert.doesNotMatch(header, /Palette/);
  assert.doesNotMatch(header, /Settings2/);
  assert.match(windowControls, /minimizeWindow/);
  assert.match(windowControls, /maximizeWindow/);
  assert.match(windowControls, /closeWindow/);
});

test('portal shell sidebar supports collapse, expand, and resize parity', () => {
  const sidebar = read('packages/sdkwork-router-portal-core/src/components/Sidebar.tsx');
  const store = read('packages/sdkwork-router-portal-core/src/store/usePortalShellStore.ts');
  const configCenter = read('packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx');

  assert.match(store, /isSidebarCollapsed/);
  assert.match(store, /sidebarWidth/);
  assert.match(store, /hiddenSidebarItems/);
  assert.match(sidebar, /PanelLeftClose|ChevronsLeft|SidebarLeft/);
  assert.match(sidebar, /PanelLeftOpen|ChevronsRight|SidebarRight/);
  assert.match(sidebar, /cursor-col-resize/);
  assert.match(sidebar, /toggleSidebar/);
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
  assert.match(profileDock, /workspace\?\.tenant\.name|workspace\?\.project\.name/);
  assert.match(profileDock, /Settings/);
  assert.match(profileDock, /Sign out|Logout/);
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
  assert.match(packageJson, /CMAKE_GENERATOR/);
  assert.match(packageJson, /Visual Studio 17 2022/);
  assert.match(packageJson, /@tauri-apps\/cli/);

  const tauriConfig = readFileSync(tauriConfigPath, 'utf8');
  assert.match(tauriConfig, /"decorations"\s*:\s*false/);
});
