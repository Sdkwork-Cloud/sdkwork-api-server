import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal app adopts the shadcn and tailwind admin-ui foundation', () => {
  const packageJson = read('package.json');
  const corePackageJson = read('packages/sdkwork-router-portal-core/package.json');
  const commonsPackageJson = read('packages/sdkwork-router-portal-commons/package.json');
  const viteConfig = read('vite.config.ts');
  const theme = read('src/theme.css');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.match(packageJson, /@tailwindcss\/vite/);
  assert.match(packageJson, /tailwindcss/);
  assert.match(packageJson, /class-variance-authority/);
  assert.match(packageJson, /clsx/);
  assert.match(packageJson, /tailwind-merge/);
  assert.match(packageJson, /lucide-react/);
  assert.match(packageJson, /@radix-ui\/react-dialog/);
  assert.match(packageJson, /@radix-ui\/react-tabs/);
  assert.match(packageJson, /recharts/);
  assert.match(packageJson, /react-router-dom/);
  assert.match(packageJson, /zustand/);
  assert.match(corePackageJson, /react-router-dom/);
  assert.match(corePackageJson, /zustand/);
  assert.match(commonsPackageJson, /"dependencies"/);
  assert.match(commonsPackageJson, /"tailwind-merge":/);
  assert.doesNotMatch(commonsPackageJson, /"devDependencies"[\s\S]*"tailwind-merge":/);

  assert.match(viteConfig, /@tailwindcss\/vite/);
  assert.match(viteConfig, /manualChunks/);
  assert.match(viteConfig, /react-vendor/);
  assert.match(viteConfig, /radix-vendor/);
  assert.match(viteConfig, /charts-vendor|icon-vendor/);
  assert.match(theme, /@import "tailwindcss";/);
  assert.match(commons, /cva\(/);
  assert.match(commons, /function cn/);
});

test('portal commons exposes shadcn-style checkbox, textarea, and modal primitives', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.match(commons, /@radix-ui\/react-checkbox/);
  assert.match(commons, /CheckboxPrimitive\.Root/);
  assert.match(commons, /export const Checkbox = forwardRef/);
  assert.match(commons, /export const Textarea = forwardRef/);
  assert.match(commons, /showCloseButton\?: boolean/);
  assert.match(commons, /export function Modal/);
  assert.match(commons, /const dialogSizeClassNames =/);
  assert.match(commons, /size = 'medium'/);
  assert.match(commons, /function DialogIconCloseButton/);
  assert.match(commons, /fixed inset-0 z-40 bg-\[var\(--portal-overlay\)\]/);
  assert.match(commons, /fixed left-1\/2 top-1\/2 z-\[60\]/);
  assert.match(commons, /DialogClose asChild>[\s\S]*?<DialogIconCloseButton/);
  assert.match(commons, /<Button/);
  assert.match(commons, /variant="ghost"/);
  assert.match(commons, /size="icon"/);
  assert.doesNotMatch(commons, /DialogClose asChild>[\s\S]*?<button/);
});

test('portal search primitives expose stable wrapper and input classes for icon-safe spacing', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.match(commons, /cn\('portalx-search-input', className\)/);
  assert.match(commons, /cn\('portalx-search-input-element', inputClassName\)/);
  assert.match(commons, /style=\{\{ \.\.\.style, paddingLeft: '2\.75rem' \}\}/);
});
