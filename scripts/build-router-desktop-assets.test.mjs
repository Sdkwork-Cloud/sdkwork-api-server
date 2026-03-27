import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('desktop asset build plan uses shell execution for pnpm on Windows', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'build-router-desktop-assets.mjs')).href
  );

  const plans = module.createDesktopAssetBuildPlan({
    workspaceRoot,
    platform: 'win32',
  });

  assert.equal(plans.length, 2);
  assert.equal(plans[0].command, 'pnpm.cmd');
  assert.equal(plans[0].shell, true);
  assert.equal(plans[1].command, 'pnpm.cmd');
  assert.equal(plans[1].shell, true);
});
