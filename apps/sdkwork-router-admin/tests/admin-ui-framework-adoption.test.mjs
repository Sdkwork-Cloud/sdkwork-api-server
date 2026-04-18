import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function readJson(relativePath) {
  return JSON.parse(read(relativePath));
}

test('app root imports sdkwork ui framework and stylesheet directly', () => {
  const main = read('src/main.tsx');
  const themeCss = read('src/theme.css');
  const packageJson = readJson('package.json');

  assert.match(main, /@sdkwork\/ui-pc-react\/styles\.css/);
  assert.match(main, /\.\/theme\.css/);
  assert.match(themeCss, /@import "tailwindcss";/);
  assert.match(themeCss, /@source "\.\/";/);
  assert.match(themeCss, /@source "\.\.\/packages";/);
  assert.match(themeCss, /\[data-theme="tech-blue"\]/);
  assert.match(themeCss, /\[data-theme="lobster"\]/);
  assert.match(themeCss, /:root\.dark/);
  assert.equal(typeof packageJson.dependencies?.['@sdkwork/ui-pc-react'], 'string');
});

test('tooling resolves the shared ui framework by package name', () => {
  const tsconfig = read('tsconfig.json');
  const viteConfig = read('vite.config.ts');

  assert.match(tsconfig, /@sdkwork\/ui-pc-react/);
  assert.match(viteConfig, /@sdkwork\\\/ui-pc-react/);
  assert.match(viteConfig, /resolveSdkworkUiSourcePath/);
});
