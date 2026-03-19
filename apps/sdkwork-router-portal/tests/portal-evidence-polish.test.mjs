import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('dashboard keeps routing evidence and recent requests visible inside the workbench flow', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.match(dashboardPage, /Routing evidence/);
  assert.match(dashboardPage, /Recent requests/);
  assert.doesNotMatch(dashboardPage, /Evidence timeline/);
  assert.doesNotMatch(dashboardPage, /Confidence signals/);
});
