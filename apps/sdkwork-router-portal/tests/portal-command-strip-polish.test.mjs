import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal shell replaces the mission strip with compact workspace navigation', () => {
  const sidebar = read('packages/sdkwork-router-portal-core/src/components/Sidebar.tsx');
  const profileDock = read('packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx');
  const shellStatus = read('packages/sdkwork-router-portal-core/src/components/ShellStatus.tsx');

  assert.doesNotMatch(sidebar, /Active workspace/);
  assert.match(shellStatus, /Workspace status/);
  assert.match(sidebar, /SidebarProfileDock/);
  assert.match(sidebar, /routeGroups\.map/);
  assert.match(profileDock, /Active workspace/);
  assert.match(profileDock, /Settings/);
  assert.match(profileDock, /Sign out/);
  assert.doesNotMatch(sidebar, /Mission strip/);
  assert.doesNotMatch(shellStatus, /Primary mission/);
});
