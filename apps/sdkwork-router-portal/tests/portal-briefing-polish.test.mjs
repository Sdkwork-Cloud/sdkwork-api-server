import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal shell removes daily-brief storytelling to keep navigation calm', () => {
  const core = read('packages/sdkwork-router-portal-core/src/index.tsx');

  assert.doesNotMatch(core, /Daily brief/);
  assert.doesNotMatch(core, /Top focus/);
  assert.doesNotMatch(core, /Risk watch/);
});

test('dashboard leads with traffic and action surfaces instead of briefing cards', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.match(dashboardPage, /Traffic overview/);
  assert.match(dashboardPage, /Quick actions/);
  assert.doesNotMatch(dashboardPage, /Focus board/);
  assert.doesNotMatch(dashboardPage, /Risk watchlist/);
});
