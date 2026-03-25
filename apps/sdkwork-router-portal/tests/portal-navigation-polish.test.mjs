import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal shell keeps sidebar navigation compact and product-led', () => {
  const sidebar = read('packages/sdkwork-router-portal-core/src/components/Sidebar.tsx');
  const profileDock = read('packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx');
  const routes = read('packages/sdkwork-router-portal-core/src/routes.ts');

  assert.doesNotMatch(sidebar, /Active workspace/);
  assert.match(sidebar, /routeGroups\.map/);
  assert.match(sidebar, /resolvePortalPath/);
  assert.match(sidebar, /Collapse sidebar|Expand sidebar/);
  assert.match(profileDock, /data-slot="portal-sidebar-footer-settings"/);
  assert.match(profileDock, /data-slot="portal-sidebar-user-control"/);
  assert.doesNotMatch(profileDock, /Active workspace/);
  assert.match(routes, /Dashboard/);
  assert.match(routes, /Routing/);
  assert.match(routes, /API Keys/);
  assert.doesNotMatch(sidebar, /Route signals/);
});

test('dashboard exposes module posture instead of a route-signal map', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.match(dashboardPage, /Module posture/);
  assert.doesNotMatch(dashboardPage, /Route signal map/);
});
