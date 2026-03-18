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

  assert.match(viteConfig, /@tailwindcss\/vite/);
  assert.match(theme, /@import "tailwindcss";/);
  assert.match(commons, /cva\(/);
  assert.match(commons, /function cn/);
});
