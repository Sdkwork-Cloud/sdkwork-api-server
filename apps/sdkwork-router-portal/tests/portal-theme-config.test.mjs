import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal theme system exposes claw-style theme mode and color controls', () => {
  const theme = read('src/theme.css');
  const themeManager = read('packages/sdkwork-router-portal-core/src/application/providers/ThemeManager.tsx');
  const store = read('packages/sdkwork-router-portal-core/src/store/usePortalShellStore.ts');

  assert.match(themeManager, /data-theme/);
  assert.match(themeManager, /themeMode/);
  assert.match(themeManager, /themeColor/);
  assert.match(store, /themeMode/);
  assert.match(store, /themeColor/);
  assert.match(store, /tech-blue/);
  assert.match(store, /lobster/);
  assert.match(store, /green-tech/);
  assert.match(store, /zinc/);
  assert.match(store, /violet/);
  assert.match(store, /rose/);
  assert.match(theme, /\[data-theme="tech-blue"\]/);
  assert.match(theme, /\[data-theme="lobster"\]/);
  assert.match(theme, /\[data-theme="green-tech"\]/);
  assert.match(theme, /\[data-theme="zinc"\]/);
  assert.match(theme, /\[data-theme="violet"\]/);
  assert.match(theme, /\[data-theme="rose"\]/);
});

test('portal preferences persist under a dedicated shell storage key', () => {
  const preferences = read('packages/sdkwork-router-portal-core/src/lib/portalPreferences.ts');
  const store = read('packages/sdkwork-router-portal-core/src/store/usePortalShellStore.ts');
  const configCenter = read('packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx');

  assert.match(preferences, /sdkwork-router-portal\.preferences\.v1/);
  assert.match(store, /persist/);
  assert.match(configCenter, /Workspace shell/);
  assert.match(configCenter, /Appearance/);
  assert.match(configCenter, /Theme mode/);
  assert.match(configCenter, /Theme color/);
  assert.match(configCenter, /Sun/);
  assert.match(configCenter, /Moon/);
  assert.match(configCenter, /Laptop/);
  assert.match(configCenter, /Sidebar navigation/);
  assert.match(configCenter, /max-h-\[calc\(100dvh-2rem\)\]/);
  assert.match(configCenter, /overflow-y-auto/);
});

test('portal default theme behavior matches claw-studio defaults', () => {
  const store = read('packages/sdkwork-router-portal-core/src/store/usePortalShellStore.ts');

  assert.match(store, /themeMode:\s*'system'/);
  assert.match(store, /themeColor:\s*'lobster'/);
  assert.doesNotMatch(store, /themeMode:\s*'dark'/);
  assert.doesNotMatch(store, /themeColor:\s*'tech-blue'/);
});

test('portal theme contract drives shell, content, and chart surfaces through shared tokens', () => {
  const theme = read('src/theme.css');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.match(theme, /--portal-shell-background/);
  assert.match(theme, /--portal-content-background/);
  assert.match(theme, /--portal-surface-background/);
  assert.match(theme, /--portal-surface-elevated/);
  assert.match(theme, /--portal-sidebar-background/);
  assert.match(theme, /--portal-border-color/);
  assert.match(theme, /--portal-text-primary/);
  assert.match(theme, /--portal-text-secondary/);
  assert.match(theme, /--portal-chart-grid/);
  assert.match(theme, /--portal-chart-tooltip-background/);
  assert.match(commons, /var\(--portal-surface-background\)/);
  assert.match(commons, /var\(--portal-text-primary\)/);
  assert.match(commons, /var\(--portal-border-color\)/);
  assert.match(dashboardPage, /var\(--portal-chart-grid\)|portal-shell-chart-surface/);
  assert.doesNotMatch(dashboardPage, /bg-slate-950\/70/);
});

test('portal shell applies gradient theme tokens through background shorthand instead of background-color utilities', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const layout = read('packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx');
  const routes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');
  const shellStatus = read('packages/sdkwork-router-portal-core/src/components/ShellStatus.tsx');
  const sidebar = read('packages/sdkwork-router-portal-core/src/components/Sidebar.tsx');

  assert.match(commons, /\[background:var\(--portal-surface-contrast\)\]/);
  assert.match(layout, /\[background:var\(--portal-shell-background\)\]/);
  assert.match(routes, /\[background:var\(--portal-surface-contrast\)\]/);
  assert.match(routes, /\[background:var\(--portal-shell-background\)\]/);
  assert.match(shellStatus, /\[background:var\(--portal-surface-contrast\)\]/);
  assert.match(sidebar, /\[background:var\(--portal-sidebar-background\)\]/);
  assert.match(sidebar, /\[background:var\(--portal-surface-contrast\)\]/);

  assert.doesNotMatch(commons, /bg-\[var\(--portal-surface-contrast\)\]/);
  assert.doesNotMatch(layout, /bg-\[var\(--portal-shell-background\)\]/);
  assert.doesNotMatch(routes, /bg-\[var\(--portal-surface-contrast\)\]/);
  assert.doesNotMatch(routes, /bg-\[var\(--portal-shell-background\)\]/);
  assert.doesNotMatch(shellStatus, /bg-\[var\(--portal-surface-contrast\)\]/);
  assert.doesNotMatch(sidebar, /bg-\[var\(--portal-sidebar-background\)\]/);
  assert.doesNotMatch(sidebar, /bg-\[var\(--portal-surface-contrast\)\]/);
});

test('portal auth shell keeps hero glow, story cards, and launch cues on shared theme tokens', () => {
  const theme = read('src/theme.css');

  assert.match(
    theme,
    /\.portalx-auth-story-card\s*\{[\s\S]*background:\s*var\(--portal-surface-elevated\);[\s\S]*border:\s*1px solid var\(--portal-border-color\);[\s\S]*color:\s*var\(--portal-text-primary\);/,
  );
  assert.match(
    theme,
    /\.portalx-auth-hero::after\s*\{[\s\S]*rgb\(var\(--portal-accent-rgb\) \/ 0\.2\)[\s\S]*transparent 60%\)\s*;/,
  );
  assert.match(
    theme,
    /\.portalx-launch-list li,\s*\.portalx-trust-list li,\s*\.portalx-help-list li\s*\{[\s\S]*border-bottom:\s*1px solid var\(--portal-border-color\);/,
  );
  assert.match(
    theme,
    /\.portalx-launch-list span:first-child\s*\{[\s\S]*background:\s*rgb\(var\(--portal-accent-rgb\) \/ 0\.14\);[\s\S]*color:\s*rgb\(var\(--portal-accent-strong-rgb\) \/ 0\.98\);/,
  );
  assert.match(
    theme,
    /\.portalx-launch-list p,\s*\.portalx-trust-list li,\s*\.portalx-help-list li\s*\{[\s\S]*color:\s*var\(--portal-text-secondary\);/,
  );
});

test('portal shell background gradients stay theme-driven instead of mixing fixed legacy accent colors', () => {
  const theme = read('src/theme.css');

  assert.match(
    theme,
    /:root\s*\{[^}]*--portal-shell-background:[^;]*var\(--portal-accent-rgb\)[^;]*var\(--portal-accent-strong-rgb\)[^;]*;/s,
  );
  assert.match(
    theme,
    /:root\.dark\s*\{[^}]*--portal-shell-background:[^;]*var\(--portal-accent-rgb\)[^;]*var\(--portal-accent-strong-rgb\)[^;]*;/s,
  );
  assert.match(theme, /:root\.dark body\s*\{[^}]*background:\s*var\(\s*--portal-shell-background\b/s);
});

test('portal desktop chrome uses dedicated titlebar and profile-dock tokens', () => {
  const theme = read('src/theme.css');
  const header = read('packages/sdkwork-router-portal-core/src/components/AppHeader.tsx');
  const windowControls = read('packages/sdkwork-router-portal-core/src/components/WindowControls.tsx');
  const profileDock = read('packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx');

  assert.match(theme, /--portal-titlebar-border/);
  assert.match(theme, /--portal-window-control-hover/);
  assert.match(theme, /--portal-window-control-danger-hover/);
  assert.match(theme, /--portal-sidebar-dock-surface/);
  assert.match(theme, /--portal-sidebar-dock-hover/);
  assert.match(theme, /--portal-sidebar-dock-panel/);
  assert.match(theme, /--portal-sidebar-dock-panel-accent/);
  assert.match(theme, /--portal-sidebar-dock-border/);
  assert.match(theme, /--portal-sidebar-dock-muted/);

  assert.match(header, /var\(--portal-titlebar-border\)/);
  assert.match(windowControls, /var\(--portal-window-control-hover\)/);
  assert.match(windowControls, /var\(--portal-window-control-danger-hover\)/);
  assert.match(profileDock, /var\(--portal-sidebar-dock-surface\)/);
  assert.match(profileDock, /var\(--portal-sidebar-dock-hover\)/);
  assert.match(profileDock, /var\(--portal-sidebar-dock-panel\)/);
  assert.match(profileDock, /var\(--portal-sidebar-dock-panel-accent\)/);
  assert.match(profileDock, /var\(--portal-sidebar-dock-border\)/);
  assert.match(profileDock, /var\(--portal-sidebar-dock-muted\)/);

  assert.doesNotMatch(windowControls, /hover:bg-white\/8/);
  assert.doesNotMatch(windowControls, /hover:bg-rose-500\/90/);
  assert.doesNotMatch(profileDock, /rgba\(244,63,94/);
  assert.doesNotMatch(profileDock, /rgba\(255,255,255,0\.04\)/);
});
