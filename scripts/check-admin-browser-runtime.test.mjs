import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('admin browser runtime smoke exposes a parseable preview build plan', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-admin-browser-runtime.mjs')).href,
  );

  assert.equal(typeof module.createAdminBrowserRuntimeSmokePlan, 'function');
  assert.equal(typeof module.runAdminBrowserRuntimeSmoke, 'function');

  const plan = module.createAdminBrowserRuntimeSmokePlan({
    workspaceRoot,
    adminAppDir: path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
    platform: 'win32',
    env: {},
    previewPort: 4173,
  });

  assert.equal(plan.previewUrl, 'http://127.0.0.1:4173/admin/');
  assert.deepEqual(plan.expectedSelectors, [
    'input[type="email"]',
    'input[type="password"]',
    'button[type="submit"]',
  ]);
  assert.match(plan.buildStep.args.join(' '), /['"]build['"]$/);
  assert.match(plan.previewStep.args.join(' '), /run-vite-cli\.mjs preview/);
  assert.match(plan.previewStep.args.join(' '), /--port 4173/);
});
