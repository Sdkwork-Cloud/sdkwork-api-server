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
  assert.match(preferences, /PORTAL_COLLAPSED_SIDEBAR_WIDTH = 72/);
  assert.match(preferences, /PORTAL_MIN_SIDEBAR_WIDTH = 220/);
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

test('portal config center mirrors claw-studio settings shell navigation and live preview structure', () => {
  const configCenter = read('packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx');

  assert.match(configCenter, /Search settings/);
  assert.match(configCenter, /appearance/);
  assert.match(configCenter, /navigation/);
  assert.match(configCenter, /workspace/);
  assert.match(configCenter, /Theme preview|Shell preview/);
  assert.match(configCenter, /Current theme/);
  assert.match(configCenter, /Restore defaults/);
  assert.match(configCenter, /Sidebar behavior/);
  assert.match(configCenter, /Theme palette|Theme color/);
  assert.match(configCenter, /SettingsSection|ThemeOptionButton|ThemeColorButton/);
  assert.match(configCenter, /flex h-full min-h-\[760px\] bg-zinc-50\/50 dark:bg-zinc-950\/50/);
  assert.match(
    configCenter,
    /flex w-72 shrink-0 flex-col border-r border-zinc-200 bg-zinc-50\/80 backdrop-blur-xl dark:border-zinc-800 dark:bg-zinc-900\/80/,
  );
  assert.match(configCenter, /scrollbar-hide flex-1 overflow-x-hidden overflow-y-auto/);
  assert.match(configCenter, /mx-auto w-full max-w-5xl p-8 md:p-12/);
  assert.match(
    configCenter,
    /border-zinc-200\/50 bg-white text-primary-600 shadow-sm dark:border-zinc-700\/50 dark:bg-zinc-800 dark:text-primary-400/,
  );
  assert.match(
    configCenter,
    /overflow-hidden rounded-\[1\.5rem\] border border-zinc-200\/80 bg-white shadow-sm transition-shadow duration-300 hover:shadow-md dark:border-zinc-800\/80 dark:bg-zinc-900/,
  );
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

test('portal shell keeps theme-driven surfaces while using claw-style raw rail chrome', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const layout = read('packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx');
  const shellStatus = read('packages/sdkwork-router-portal-core/src/components/ShellStatus.tsx');
  const sidebar = read('packages/sdkwork-router-portal-core/src/components/Sidebar.tsx');

  assert.match(commons, /\[background:var\(--portal-surface-contrast\)\]/);
  assert.match(layout, /\[background:var\(--portal-shell-background\)\]/);
  assert.match(shellStatus, /\[background:var\(--portal-surface-contrast\)\]/);
  assert.match(sidebar, /bg-\[linear-gradient\(180deg,_#13151a_0%,_#0b0c10_100%\)\]/);
  assert.match(sidebar, /bg-zinc-950/);

  assert.doesNotMatch(commons, /bg-\[var\(--portal-surface-contrast\)\]/);
  assert.doesNotMatch(layout, /bg-\[var\(--portal-shell-background\)\]/);
  assert.doesNotMatch(shellStatus, /bg-\[var\(--portal-surface-contrast\)\]/);
  assert.doesNotMatch(sidebar, /\[background:var\(--portal-sidebar-background\)\]/);
});

test('portal authenticated shell removes the global ShellStatus banner and keeps the right content area at maximum usable width', () => {
  const layout = read('packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx');
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');
  const app = read('packages/sdkwork-router-portal-core/src/application/app/PortalProductApp.tsx');

  assert.doesNotMatch(layout, /ShellStatus/);
  assert.doesNotMatch(layout, /max-w-\[1600px\]/);
  assert.match(layout, /min-h-full w-full flex-col gap-6 px-4 py-5 md:px-6 xl:px-8/);
  assert.doesNotMatch(appRoutes, /pulseDetail|pulseStatus|pulseTitle|pulseTone/);
  assert.doesNotMatch(app, /buildWorkspacePulse|pulseDetail|pulseStatus|pulseTitle|pulseTone/);
});

test('portal auth page mirrors claw-studio surfaces while still honoring theme mode and theme color accents', () => {
  const authPage = read('packages/sdkwork-router-portal-auth/src/pages/AuthPage.tsx');

  assert.match(authPage, /bg-zinc-50/);
  assert.match(authPage, /dark:bg-zinc-950/);
  assert.match(authPage, /bg-zinc-900/);
  assert.match(authPage, /dark:bg-black/);
  assert.match(authPage, /from-primary-600\/20/);
  assert.match(authPage, /text-primary-600/);
  assert.match(authPage, /hover:text-primary-500/);
  assert.match(authPage, /dark:border-zinc-800/);
  assert.match(authPage, /dark:bg-zinc-900/);
  assert.doesNotMatch(authPage, /AuthShell/);
  assert.doesNotMatch(authPage, /portalx-auth-hero/);
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
  assert.match(theme, /body\s*\{[^}]*background:\s*var\(\s*--portal-shell-background\b/s);
});

test('portal theme substrate matches claw-studio scrollbar and dark color-scheme behavior', () => {
  const theme = read('src/theme.css');

  assert.match(theme, /--scrollbar-size: 10px/);
  assert.match(theme, /--scrollbar-track:/);
  assert.match(theme, /--scrollbar-thumb:/);
  assert.match(theme, /scrollbar-width: thin/);
  assert.match(theme, /scrollbar-color: var\(--scrollbar-thumb\) var\(--scrollbar-track\)/);
  assert.match(theme, /:root\.dark\s*\{[^}]*color-scheme:\s*dark/s);
});

test('portal desktop chrome uses claw-style glass titlebar and account-control surfaces', () => {
  const header = read('packages/sdkwork-router-portal-core/src/components/AppHeader.tsx');
  const windowControls = read('packages/sdkwork-router-portal-core/src/components/WindowControls.tsx');
  const profileDock = read('packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx');

  assert.match(header, /bg-white\/72 backdrop-blur-xl dark:bg-zinc-950\/78/);
  assert.match(windowControls, /hover:bg-zinc-950\/\[0\.06\]/);
  assert.match(windowControls, /hover:bg-rose-500 hover:text-white/);
  assert.match(profileDock, /border-white\/8 bg-white\/\[0\.04\]/);
  assert.match(profileDock, /border-white\/10 bg-zinc-950\/96/);
  assert.match(profileDock, /bg-primary-500\/15[\s\S]*text-primary-200/);
  assert.doesNotMatch(profileDock, /var\(--portal-sidebar-dock-surface\)/);
  assert.doesNotMatch(windowControls, /var\(--portal-window-control-hover\)/);
});
