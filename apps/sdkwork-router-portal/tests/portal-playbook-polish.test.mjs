import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal shell removes operating-rhythm storytelling from the sidebar', () => {
  const core = read('packages/sdkwork-router-portal-core/src/index.tsx');

  assert.doesNotMatch(core, /Operating rhythm/);
  assert.doesNotMatch(core, /Before traffic/);
  assert.doesNotMatch(core, /If risk appears/);
});

test('dashboard removes playbook-era sections in favor of direct overview modules', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.doesNotMatch(dashboardPage, /Review cadence/);
  assert.doesNotMatch(dashboardPage, /Playbook lane/);
  assert.doesNotMatch(dashboardPage, /Action queue/);
  assert.doesNotMatch(dashboardPage, /Production readiness/);
  assert.doesNotMatch(dashboardPage, /Launch checklist/);
});
