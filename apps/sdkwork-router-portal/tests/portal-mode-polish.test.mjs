import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('dashboard turns abstract mode guidance into provider and model demand views', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.match(dashboardPage, /Provider share/);
  assert.match(dashboardPage, /Model demand/);
  assert.doesNotMatch(dashboardPage, /Mode narrative/);
  assert.doesNotMatch(dashboardPage, /Decision path/);
});
