import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

function writeFixtureApp({ root, relativeAppDir, entryJs, entryCss, extraAssets = {} }) {
  const appRoot = path.join(root, relativeAppDir);
  const distRoot = path.join(appRoot, 'dist');
  const assetsRoot = path.join(distRoot, 'assets');

  mkdirSync(assetsRoot, { recursive: true });
  writeFileSync(
    path.join(distRoot, 'index.html'),
    [
      '<!doctype html>',
      '<html>',
      '  <head>',
      `    <link rel="stylesheet" crossorigin href="/assets/${entryCss}">`,
      '  </head>',
      '  <body>',
      `    <script type="module" crossorigin src="/assets/${entryJs}"></script>`,
      '  </body>',
      '</html>',
      '',
    ].join('\n'),
    'utf8',
  );
  writeFileSync(path.join(assetsRoot, entryJs), 'export const ready = true;\n', 'utf8');
  writeFileSync(path.join(assetsRoot, entryCss), 'body{margin:0;}\n', 'utf8');

  for (const [assetName, contents] of Object.entries(extraAssets)) {
    writeFileSync(path.join(assetsRoot, assetName), contents, 'utf8');
  }
}

test('desktop asset build plan uses the shared hidden Windows pnpm launcher', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'build-router-desktop-assets.mjs')).href
  );

  const plans = module.createDesktopAssetBuildPlan({
    workspaceRoot,
    platform: 'win32',
  });

  assert.equal(plans.length, 2);
  assert.equal(plans[0].command, 'powershell.exe');
  assert.match(plans[0].args.join(' '), /pnpm\.cjs/);
  assert.match(plans[0].args.join(' '), /build/);
  assert.equal(plans[0].shell, false);
  assert.equal(plans[1].command, 'powershell.exe');
  assert.match(plans[1].args.join(' '), /pnpm\.cjs/);
  assert.match(plans[1].args.join(' '), /build/);
  assert.equal(plans[1].shell, false);
});

test('desktop asset build exposes a post-build budget gate for release frontend bundles', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'build-router-desktop-assets.mjs')).href
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-desktop-asset-budget-'));
  writeFixtureApp({
    root: fixtureRoot,
    relativeAppDir: 'apps/sdkwork-router-admin',
    entryJs: 'index-admin.js',
    entryCss: 'index-admin.css',
    extraAssets: {
      'react-vendor-admin.js': 'export const reactVendor = true;\n',
      'async-admin.js': 'export const asyncChunk = true;\n',
    },
  });
  writeFixtureApp({
    root: fixtureRoot,
    relativeAppDir: 'apps/sdkwork-router-portal',
    entryJs: 'index-portal.js',
    entryCss: 'index-portal.css',
    extraAssets: {
      'react-vendor-portal.js': 'export const reactVendor = true;\n',
      'async-portal.js': 'export const asyncChunk = true;\n',
    },
  });

  const oversizedEntry = [
    'export const oversized = `',
    'x'.repeat(500_000),
    '`;\n',
  ].join('');
  writeFileSync(
    path.join(fixtureRoot, 'apps', 'sdkwork-router-admin', 'dist', 'assets', 'index-admin.js'),
    oversizedEntry,
    'utf8',
  );

  await assert.rejects(
    module.runPostBuildChecks({
      workspaceRoot: fixtureRoot,
    }),
    /admin entry/i,
  );
});
