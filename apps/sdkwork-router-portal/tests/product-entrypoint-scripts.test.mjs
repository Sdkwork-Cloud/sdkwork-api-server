import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const appRoot = path.resolve(import.meta.dirname, '..');
const workspaceRoot = path.resolve(appRoot, '..', '..');

test('portal package exposes product-grade server plan and integrated product checks', async () => {
  const packageJson = await import(pathToFileURL(path.join(appRoot, 'package.json')).href, {
    with: { type: 'json' },
  });

  assert.equal(
    packageJson.default.scripts['product:start'],
    'node ../../scripts/run-router-product.mjs',
  );
  assert.equal(
    packageJson.default.scripts['server:plan'],
    'node ../../scripts/run-router-product-service.mjs --dry-run --plan-format json',
  );
  assert.equal(
    packageJson.default.scripts['product:check'],
    'node ../../scripts/check-router-product.mjs',
  );
});

test('product check script plans portal and admin regression tests before build and server verification', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-router-product.mjs')).href
  );

  const plan = module.createProductCheckPlan({
    workspaceRoot,
    portalAppDir: appRoot,
    adminAppDir: path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
    platform: 'win32',
    env: {},
  });

  assert.equal(plan.length, 9);
  assert.equal(plan[0].label, 'portal typecheck');
  assert.equal(plan[1].label, 'portal regression tests');
  assert.equal(plan[2].label, 'admin typecheck');
  assert.equal(plan[3].label, 'admin regression tests');
  assert.equal(plan[4].label, 'docs bootstrap safety');
  assert.equal(plan[5].label, 'workspace dependency audit');
  assert.equal(plan[6].label, 'desktop assets build');
  assert.equal(plan[7].label, 'server cargo check');
  assert.equal(plan[8].label, 'server deployment plan');
  assert.deepEqual(plan[1].args, ['--test', 'tests/*.mjs']);
  assert.deepEqual(plan[3].args, ['--test', 'tests/*.mjs']);
  assert.match(plan[4].args.join(' '), /check-router-docs-safety\.mjs/);
  assert.match(plan[5].args.join(' '), /check-rust-dependency-audit\.mjs/);
  assert.match(plan[8].args.join(' '), /--dry-run/);
  assert.match(plan[8].args.join(' '), /--plan-format json/);
});
