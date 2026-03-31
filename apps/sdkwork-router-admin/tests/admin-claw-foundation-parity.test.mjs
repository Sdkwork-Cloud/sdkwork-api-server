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

test('admin shell stylesheet carries the claw-studio theme token and scrollbar primitives', () => {
  const adminTheme = readFromApp('packages/sdkwork-router-admin-shell/src/styles/index.css');
  const clawTheme = readFromClaw('packages/sdkwork-claw-shell/src/styles/index.css');

  const requiredSnippets = [
    '--theme-primary-50: #eff6ff;',
    '--theme-primary-500: #3b82f6;',
    '[data-theme="lobster"]',
    '[data-theme="green-tech"]',
    '[data-theme="zinc"]',
    '[data-theme="violet"]',
    '[data-theme="rose"]',
    '.custom-scrollbar {',
    '.scrollbar-hide {',
  ];

  for (const snippet of requiredSnippets) {
    assert.match(clawTheme, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
    assert.match(adminTheme, new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }

  assert.match(clawTheme, /@source "\.\.\/\.\.\/\.\.\/\.\.\/";/);
  assert.match(adminTheme, /@source "\.\.\/\.\.\/\.\.\/\.\.\/src";/);
  assert.match(adminTheme, /@source "\.\.\/\.\.\/\.\.\/";/);
  assert.doesNotMatch(adminTheme, /@source "\.\.\/\.\.\/\.\.\/\.\.\/";/);
  assert.match(adminTheme, /\.ghost-scrollbar \{/);
});

test('admin commons primitives use claw-style tailwind surfaces instead of admin-only shell wrappers', () => {
  const commons = readFromApp('packages/sdkwork-router-admin-commons/src/index.tsx');

  assert.match(commons, /rounded-3xl border/);
  assert.match(commons, /rounded-\[28px\] border border-zinc-200\/80 bg-white\/92/);
  assert.match(commons, /sticky top-0 z-10 whitespace-nowrap border-b border-zinc-200\/80/);
  assert.match(commons, /transition-colors hover:bg-zinc-50\/80/);
  assert.doesNotMatch(commons, /adminx-page-toolbar/);
  assert.doesNotMatch(commons, /adminx-surface/);
  assert.doesNotMatch(commons, /adminx-stat-card/);
  assert.doesNotMatch(commons, /adminx-table-wrap/);
  assert.doesNotMatch(commons, /adminx-table-row/);
});

test('admin commons exposes shadcn-style checkbox, textarea, and modal primitives for unified dialog forms', () => {
  const commons = readFromApp('packages/sdkwork-router-admin-commons/src/index.tsx');

  assert.match(commons, /@radix-ui\/react-checkbox/);
  assert.match(commons, /CheckboxPrimitive\.Root/);
  assert.match(commons, /export const Checkbox = forwardRef/);
  assert.match(commons, /export const Textarea = forwardRef/);
  assert.match(commons, /min-h-\[96px\]/);
  assert.match(commons, /export function Modal/);
  assert.match(commons, /showCloseButton=\{false\}/);
});

test('admin modal chrome reuses shared Button primitives instead of private close-button markup', () => {
  const commons = readFromApp('packages/sdkwork-router-admin-commons/src/index.tsx');

  assert.match(commons, /function DialogIconCloseButton/);
  assert.match(commons, /DialogClose asChild>[\s\S]*?<DialogIconCloseButton/);
  assert.match(commons, /<Button/);
  assert.match(commons, /variant="ghost"/);
  assert.match(commons, /size="icon"/);
  assert.doesNotMatch(commons, /DialogClose asChild>[\s\S]*?<button/);
});

test('admin modal forms reuse shared Textarea instead of raw textarea tags', () => {
  const files = [
    'packages/sdkwork-router-admin-coupons/src/index.tsx',
    'packages/sdkwork-router-admin-tenants/src/index.tsx',
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayModelMappingsPage.tsx',
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx',
  ];

  for (const relativePath of files) {
    const source = readFromApp(relativePath);

    assert.match(source, /Textarea/);
    assert.doesNotMatch(source, /<textarea/);
  }
});

test('admin shell chrome uses claw-style translucent header and dark rail classes directly in React', () => {
  const header = readFromApp('packages/sdkwork-router-admin-shell/src/components/AppHeader.tsx');
  const sidebar = readFromApp('packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx');

  assert.match(header, /bg-white\/72 backdrop-blur-xl dark:bg-zinc-950\/78/);
  assert.match(header, /rounded-2xl bg-zinc-950\/\[\d\.\d+\]/);
  assert.match(sidebar, /bg-\[linear-gradient\(180deg,_#13151a_0%,_#0b0c10_100%\)\]/);
  assert.match(sidebar, /shadow-\[18px_0_50px_rgba\(9,9,11,0\.16\)\]/);
  assert.match(sidebar, /text-\[14px\] tracking-tight/);
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
