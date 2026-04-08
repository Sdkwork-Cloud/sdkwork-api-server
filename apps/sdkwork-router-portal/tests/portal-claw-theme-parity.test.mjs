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

test('portal theme stylesheet keeps the claw-studio token and scrollbar foundation intact', () => {
  const portalTheme = readFromApp('src/theme.css');
  const clawTheme = readFromClaw('packages/sdkwork-claw-shell/src/styles/index.css');
  const clawWorkspaceSourceDirectivePattern = /@source "(?:\.\.\/){3,}";/;

  const requiredThemeSnippets = [
    '--theme-primary-50: #eff6ff;',
    '--theme-primary-500: #3b82f6;',
    '[data-theme="lobster"]',
    '[data-theme="green-tech"]',
    '[data-theme="zinc"]',
    '[data-theme="violet"]',
    '[data-theme="rose"]',
    '--scrollbar-track: color-mix(in srgb, var(--theme-primary-200) 10%, transparent);',
    '.custom-scrollbar {',
    '.scrollbar-hide {',
    '@keyframes shake',
  ];

  for (const snippet of requiredThemeSnippets) {
    assert.match(clawTheme, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
    assert.match(portalTheme, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }

  assert.match(clawTheme, clawWorkspaceSourceDirectivePattern);
  assert.match(portalTheme, /@source "\.\/";/);
  assert.match(portalTheme, /@source "\.\.\/packages";/);
  assert.doesNotMatch(portalTheme, clawWorkspaceSourceDirectivePattern);
  assert.match(portalTheme, /\.ghost-scrollbar \{/);
});

test('portal ThemeManager follows the claw-studio root contract instead of router-local theme flags', () => {
  const themeManager = readFromApp(
    'packages/sdkwork-router-portal-core/src/application/providers/ThemeManager.tsx',
  );

  assert.match(themeManager, /root\.setAttribute\('data-theme', themeColor\)/);
  assert.match(themeManager, /SdkworkThemeProvider/);
  assert.match(themeManager, /createSdkworkTheme/);
  assert.match(themeManager, /root\.classList\.toggle\('dark', resolvedColorMode === 'dark'\)/);
  assert.match(themeManager, /window\.matchMedia\('\(prefers-color-scheme: dark\)'\)/);
  assert.doesNotMatch(themeManager, /data-theme-mode/);
  assert.doesNotMatch(themeManager, /data-sidebar-collapsed/);
  assert.doesNotMatch(themeManager, /theme-dark/);
  assert.doesNotMatch(themeManager, /theme-light/);
  assert.doesNotMatch(themeManager, /root\.classList\.add\('dark'\)/);
  assert.doesNotMatch(themeManager, /root\.classList\.remove\('dark'\)/);
});

test('portal sidebar collapse heuristics and persisted preference mirror claw-studio', () => {
  const portalStore = readFromApp('packages/sdkwork-router-portal-core/src/store/usePortalShellStore.ts');
  const portalAutoCollapse = readFromApp(
    'packages/sdkwork-router-portal-core/src/lib/sidebarAutoCollapse.ts',
  );
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
    assert.match(portalStore, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }

  const portalAutoCollapseSnippets = [
    'const COMPACT_VIEWPORT_WIDTH = 1440;',
    'const ROOMY_VIEWPORT_WIDTH = 1600;',
    'const TIGHT_VIEWPORT_HEIGHT = 900;',
    'const HIGH_SCALE_FACTOR = 1.25;',
    'const TIGHT_EFFECTIVE_SCREEN_HEIGHT = 920;',
    'export function shouldAutoCollapseSidebar',
    'export function resolveAutoSidebarCollapsed',
  ];

  for (const snippet of portalAutoCollapseSnippets) {
    assert.match(portalAutoCollapse, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }

  const portalStoreSnippets = [
    'sidebarCollapsePreference',
    "sidebarCollapsePreference: 'auto'",
    'resolveAutoSidebarCollapsed()',
    "sidebarCollapsePreference: 'user'",
    "sidebarCollapsePreference === 'auto'",
  ];

  for (const snippet of portalStoreSnippets) {
    assert.match(portalStore, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }
});
