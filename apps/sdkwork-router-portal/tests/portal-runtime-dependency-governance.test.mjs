import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal root package pins a single runtime dependency baseline for the workspace', () => {
  const packageJson = JSON.parse(read('package.json'));

  assert.equal(packageJson.pnpm?.overrides?.react, '19.2.4');
  assert.equal(packageJson.pnpm?.overrides?.['react-dom'], '19.2.4');
  assert.equal(packageJson.pnpm?.overrides?.['react-router-dom'], '7.13.1');
  assert.equal(packageJson.pnpm?.overrides?.['react-router'], '7.13.1');
  assert.equal(packageJson.pnpm?.overrides?.['lucide-react'], '0.554.0');
  assert.equal(packageJson.pnpm?.overrides?.motion, '12.38.0');
  assert.equal(packageJson.pnpm?.overrides?.['framer-motion'], '12.38.0');
  assert.equal(packageJson.pnpm?.overrides?.zustand, '5.0.12');
  assert.equal(packageJson.pnpm?.overrides?.vite, '7.3.1');
  assert.equal(packageJson.pnpm?.overrides?.['@vitejs/plugin-react'], '5.2.0');
  assert.equal(packageJson.pnpm?.overrides?.tailwindcss, '4.2.1');
  assert.equal(packageJson.pnpm?.overrides?.['@tailwindcss/vite'], '4.2.1');
  assert.equal(packageJson.pnpm?.overrides?.typescript, '5.9.3');
});

test('vite config forces shared singleton resolution for linked framework runtime packages', () => {
  const viteConfig = read('vite.config.ts');

  assert.match(viteConfig, /dedupe:\s*\[[\s\S]*'react'[\s\S]*'react-dom'[\s\S]*'react-router-dom'[\s\S]*'lucide-react'[\s\S]*'zustand'[\s\S]*\]/);
  assert.match(viteConfig, /const sdkworkUiSourceRoot =/);
  assert.match(viteConfig, /const zustandPackageRoot = normalizeAliasPath\(resolvePnpmPackageRoot\('zustand'\)\);/);
  assert.match(viteConfig, /const zustandEsmEntry = normalizeAliasPath\(path\.join\(zustandPackageRoot, 'esm', 'index\.mjs'\)\);/);
  assert.match(viteConfig, /const zustandEsmSubpathRoot = `\$\{normalizeAliasPath\(path\.join\(zustandPackageRoot, 'esm'\)\)\}\/`;/);
  assert.match(viteConfig, /find:\s*\/\^react\$\/,/);
  assert.match(viteConfig, /find:\s*\/\^react-dom\$\/,/);
  assert.match(viteConfig, /find:\s*\/\^react-dom\\\/client\$\/,/);
  assert.match(viteConfig, /find:\s*\/\^react\\\/jsx-runtime\$\/,/);
  assert.match(viteConfig, /find:\s*\/\^react\\\/jsx-dev-runtime\$\/,/);
  assert.match(viteConfig, /find:\s*\/\^react-router-dom\$\/,/);
  assert.match(viteConfig, /find:\s*\/\^zustand\$\/,/);
  assert.match(viteConfig, /find:\s*\/\^zustand\\\/\//);
  assert.match(viteConfig, /replacement:\s*zustandEsmEntry,/);
  assert.match(viteConfig, /replacement:\s*zustandEsmSubpathRoot,/);
  assert.match(viteConfig, /find:\s*\/\^clsx\$\/,/);
  assert.match(viteConfig, /find:\s*\/\^tailwind-merge\$\/,/);
  assert.match(viteConfig, /find:\s*\/\^lucide-react\$\/,/);
  assert.match(viteConfig, /find:\s*\/\^sdkwork-router-portal-commons\\\/\//);
  assert.match(viteConfig, /find:\s*\/\^@sdkwork\\\/ui-pc-react\\\/theme\$\//);
  assert.match(viteConfig, /find:\s*\/\^@sdkwork\\\/ui-pc-react\\\/components\\\/ui\\\/actions\$\//);
  assert.match(viteConfig, /find:\s*\/\^@sdkwork\\\/ui-pc-react\\\/components\\\/ui\\\/data-display\$\//);
  assert.match(viteConfig, /find:\s*\/\^@sdkwork\\\/ui-pc-react\\\/components\\\/patterns\\\/workspace\$\//);
  assert.match(viteConfig, /theme\/index\.ts/);
  assert.match(viteConfig, /components\/ui\/actions\/index\.ts/);
  assert.match(viteConfig, /components\/ui\/data-display\/index\.ts/);
  assert.match(viteConfig, /components\/patterns\/workspace\/index\.ts/);
  assert.match(viteConfig, /src\/vendor\/lucide-react\.ts/);
  assert.match(viteConfig, /style-vendor/);
  assert.match(viteConfig, /tailwind-merge/);
  assert.match(viteConfig, /class-variance-authority/);
  assert.match(viteConfig, /clsx/);
  assert.doesNotMatch(viteConfig, /lucide-vendor/);
  assert.doesNotMatch(viteConfig, /portalUiChunkGroups/);
  assert.doesNotMatch(viteConfig, /resolvePortalUiChunk/);
  assert.doesNotMatch(viteConfig, /portal-shared-ui/);
});

test('index html advertises the polished portal shell metadata', () => {
  const indexHtml = read('index.html');

  assert.match(indexHtml, /<meta name="application-name" content="SDKWork Router Portal" \/>/);
  assert.match(indexHtml, /<meta name="description" content="SDKWork Router developer portal for gateway operations, API access, billing, and workspace management\." \/>/);
  assert.match(indexHtml, /<meta name="theme-color" content="#09090b" \/>/);
  assert.match(indexHtml, /<meta name="color-scheme" content="dark light" \/>/);
});
