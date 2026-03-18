import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('dashboard keeps routing posture and workspace modules visible as the main handoff surfaces', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.match(dashboardPage, /Routing posture/);
  assert.match(dashboardPage, /Workspace modules/);
  assert.doesNotMatch(dashboardPage, /Journey progress/);
  assert.doesNotMatch(dashboardPage, /Milestone map/);
});
