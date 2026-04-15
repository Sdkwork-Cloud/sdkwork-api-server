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
      `    <link rel="stylesheet" crossorigin href="/${relativeAppDir}/assets/${entryCss}">`,
      '  </head>',
      '  <body>',
      `    <script type="module" crossorigin src="/${relativeAppDir}/assets/${entryJs}"></script>`,
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

test('frontend budget audit passes when current app assets stay within commercial thresholds', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-router-frontend-budgets.mjs')).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-frontend-budget-pass-'));
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
      'portalMessages.zh-CN-portal.js': 'export const localeCatalog = true;\n',
    },
  });

  const report = module.evaluateFrontendBudgets({
    workspaceRoot: fixtureRoot,
  });

  assert.equal(report.ok, true);
  assert.equal(report.failures.length, 0);
});

test('frontend budget audit fails when a release entry bundle exceeds the enforced threshold', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-router-frontend-budgets.mjs')).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-frontend-budget-fail-'));
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
      'portalMessages.zh-CN-portal.js': 'export const localeCatalog = true;\n',
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

  const report = module.evaluateFrontendBudgets({
    workspaceRoot: fixtureRoot,
  });

  assert.equal(report.ok, false);
  assert.ok(
    report.failures.some((failure) => failure.ruleId === 'admin-entry-js'),
    'expected the admin entry bundle budget to fail',
  );
  assert.match(report.summary, /admin entry/i);
});

test('frontend budget audit treats the portal zh-CN locale catalog as a dedicated async budget class', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-router-frontend-budgets.mjs')).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-frontend-budget-locale-'));
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
      'portalMessages.zh-CN-portal.js': [
        'export const localeCatalog = `',
        'x'.repeat(110_000),
        '`;\n',
      ].join(''),
    },
  });

  const report = module.evaluateFrontendBudgets({
    workspaceRoot: fixtureRoot,
  });

  assert.equal(report.ok, true);
  assert.equal(report.failures.length, 0);
});
