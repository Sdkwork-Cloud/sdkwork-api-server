import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
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

test('portal theme stylesheet keeps the claw-studio token and scrollbar foundation intact', () => {
  const portalTheme = readFromApp('src/theme.css');
  const clawTheme = readFromClaw('packages/sdkwork-claw-shell/src/styles/index.css');

  const requiredThemeSnippets = [
    '@source "../../../../";',
    '--theme-primary-50: #eff6ff;',
    '--theme-primary-500: #3b82f6;',
    '[data-theme="lobster"]',
    '[data-theme="green-tech"]',
    '[data-theme="zinc"]',
    '[data-theme="violet"]',
    '[data-theme="rose"]',
    '--scrollbar-track: color-mix(in srgb, var(--theme-primary-200) 10%, transparent);',
    '.custom-scrollbar {',
    '.ghost-scrollbar {',
    '.scrollbar-hide {',
    '@keyframes shake',
  ];

  for (const snippet of requiredThemeSnippets) {
    assert.match(clawTheme, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
    assert.match(portalTheme, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }
});

test('portal ThemeManager follows the claw-studio root contract instead of router-local theme flags', () => {
  const themeManager = readFromApp(
    'packages/sdkwork-router-portal-core/src/application/providers/ThemeManager.tsx',
  );

  assert.match(themeManager, /root\.setAttribute\('data-theme', themeColor\)/);
  assert.match(themeManager, /root\.classList\.add\('dark'\)/);
  assert.match(themeManager, /root\.classList\.remove\('dark'\)/);
  assert.match(themeManager, /window\.matchMedia\('\(prefers-color-scheme: dark\)'\)/);
  assert.doesNotMatch(themeManager, /data-theme-mode/);
  assert.doesNotMatch(themeManager, /data-sidebar-collapsed/);
  assert.doesNotMatch(themeManager, /theme-dark/);
  assert.doesNotMatch(themeManager, /theme-light/);
});

test('portal sidebar collapse heuristics and persisted preference mirror claw-studio', () => {
  const portalStore = readFromApp('packages/sdkwork-router-portal-core/src/store/usePortalShellStore.ts');
  const portalAutoCollapse = readFromApp(
    'packages/sdkwork-router-portal-core/src/lib/sidebarAutoCollapse.ts',
  );
  const clawStore = readFromClaw('packages/sdkwork-claw-core/src/stores/useAppStore.ts');
  const clawAutoCollapse = readFromClaw('packages/sdkwork-claw-core/src/stores/sidebarAutoCollapse.ts');

  const sharedAutoCollapseSnippets = [
    'const COMPACT_VIEWPORT_WIDTH = 1440;',
    'const ROOMY_VIEWPORT_WIDTH = 1600;',
    'const TIGHT_VIEWPORT_HEIGHT = 900;',
    'const HIGH_SCALE_FACTOR = 1.25;',
    'const TIGHT_EFFECTIVE_SCREEN_HEIGHT = 920;',
    'export function shouldAutoCollapseSidebar',
    'export function resolveAutoSidebarCollapsed',
  ];

  for (const snippet of sharedAutoCollapseSnippets) {
    assert.match(clawAutoCollapse, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
    assert.match(portalAutoCollapse, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }

  const sharedStoreSnippets = [
    'sidebarCollapsePreference',
    "sidebarCollapsePreference: 'auto'",
    'resolveAutoSidebarCollapsed()',
    "sidebarCollapsePreference: 'user'",
    "sidebarCollapsePreference === 'auto'",
  ];

  for (const snippet of sharedStoreSnippets) {
    assert.match(clawStore, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
    assert.match(portalStore, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }
});
