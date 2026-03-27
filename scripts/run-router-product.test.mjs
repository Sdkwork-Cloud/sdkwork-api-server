import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('router product launcher preserves forwarded mode arguments after --', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'run-router-product.mjs')).href
  );

  const parsed = module.parseRouterProductArgs(['server', '--', '--help']);

  assert.equal(parsed.mode, 'server');
  assert.equal(parsed.help, false);
  assert.deepEqual(parsed.extraArgs, ['--help']);
});

test('router product launcher defaults to desktop mode and installs dependencies when requested', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'run-router-product.mjs')).href
  );

  const plan = module.createRouterProductLaunchPlan({
    workspaceRoot,
    mode: 'desktop',
    install: true,
    platform: 'win32',
    env: {},
    extraArgs: [],
  });

  assert.equal(plan.length, 2);
  assert.equal(plan[0].label, 'portal install');
  assert.deepEqual(plan[0].args, ['--dir', 'apps/sdkwork-router-portal', 'install']);
  assert.equal(plan[0].command, 'pnpm.cmd');
  assert.equal(plan[0].shell, true);
  assert.equal(plan[1].label, 'portal desktop runtime');
  assert.deepEqual(plan[1].args, ['--dir', 'apps/sdkwork-router-portal', 'tauri:dev']);
});

test('router product launcher forwards cluster arguments into server mode', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'run-router-product.mjs')).href
  );

  const plan = module.createRouterProductLaunchPlan({
    workspaceRoot,
    mode: 'server',
    install: false,
    platform: 'linux',
    env: {},
    extraArgs: ['--roles', 'web', '--gateway-upstream', '10.0.0.21:8080'],
  });

  assert.equal(plan.length, 1);
  assert.equal(plan[0].label, 'portal product server');
  assert.equal(plan[0].command, 'pnpm');
  assert.deepEqual(plan[0].args, [
    '--dir',
    'apps/sdkwork-router-portal',
    'server:start',
    '--',
    '--roles',
    'web',
    '--gateway-upstream',
    '10.0.0.21:8080',
  ]);
  assert.equal(plan[0].shell, false);
});

test('router product launcher exposes machine-readable plan mode through the unified entrypoint', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'run-router-product.mjs')).href
  );

  const plan = module.createRouterProductLaunchPlan({
    workspaceRoot,
    mode: 'plan',
    install: false,
    platform: 'linux',
    env: {},
    extraArgs: ['--roles', 'web'],
  });

  assert.equal(plan.length, 1);
  assert.equal(plan[0].label, 'portal deployment plan');
  assert.equal(plan[0].command, process.execPath);
  assert.deepEqual(plan[0].args, [
    path.join(workspaceRoot, 'scripts', 'run-router-product-service.mjs'),
    '--dry-run',
    '--plan-format',
    'json',
    '--roles',
    'web',
  ]);
});
