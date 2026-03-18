import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('shared admin commons expose page toolbar, dialog primitives, and form fields for CRUD workbenches', () => {
  const commons = read('packages/sdkwork-router-admin-commons/src/index.tsx');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(commons, /PageToolbar/);
  assert.match(commons, /Dialog/);
  assert.match(commons, /DialogContent/);
  assert.match(commons, /DialogFooter/);
  assert.match(commons, /ConfirmDialog/);
  assert.match(commons, /FormField/);
  assert.match(theme, /adminx-page-toolbar/);
  assert.match(theme, /adminx-dialog-backdrop/);
  assert.match(theme, /adminx-dialog-panel/);
  assert.match(theme, /adminx-confirm-dialog/);
});

test('users workbench separates create and edit flows into dedicated dialogs', () => {
  const users = read('packages/sdkwork-router-admin-users/src/index.tsx');

  assert.match(users, /PageToolbar/);
  assert.match(users, /Dialog/);
  assert.match(users, /DialogContent/);
  assert.match(users, /New operator/);
  assert.match(users, /New portal user/);
  assert.match(users, /ConfirmDialog/);
  assert.match(users, /Edit operator/);
  assert.match(users, /Edit portal user/);
  assert.match(users, /pendingDelete/);
});

test('tenants workbench promotes tenant, project, and key issuance into independent actions and dialogs', () => {
  const tenants = read('packages/sdkwork-router-admin-tenants/src/index.tsx');

  assert.match(tenants, /PageToolbar/);
  assert.match(tenants, /Dialog/);
  assert.match(tenants, /DialogContent/);
  assert.match(tenants, /New tenant/);
  assert.match(tenants, /New project/);
  assert.match(tenants, /Issue gateway key/);
  assert.match(tenants, /ConfirmDialog/);
  assert.match(tenants, /revealedApiKey/);
});

test('coupon workbench uses a focused campaign dialog instead of inline form editing', () => {
  const coupons = read('packages/sdkwork-router-admin-coupons/src/index.tsx');

  assert.match(coupons, /PageToolbar/);
  assert.match(coupons, /Dialog/);
  assert.match(coupons, /DialogContent/);
  assert.match(coupons, /New coupon/);
  assert.match(coupons, /ConfirmDialog/);
  assert.match(coupons, /Edit coupon campaign/);
  assert.match(coupons, /pendingDeleteCoupon/);
});

test('catalog workbench keeps registries primary and moves maintenance into dialogs', () => {
  const catalog = read('packages/sdkwork-router-admin-catalog/src/index.tsx');

  assert.match(catalog, /PageToolbar/);
  assert.match(catalog, /Dialog/);
  assert.match(catalog, /DialogContent/);
  assert.match(catalog, /ConfirmDialog/);
  assert.match(catalog, /New channel/);
  assert.match(catalog, /New provider/);
  assert.match(catalog, /Rotate credential/);
  assert.match(catalog, /New model/);
  assert.doesNotMatch(catalog, /Channel maintenance/);
  assert.doesNotMatch(catalog, /Provider maintenance/);
});

test('operations workbench uses targeted dialogs and preserves runtime status as the primary surface', () => {
  const operations = read('packages/sdkwork-router-admin-operations/src/index.tsx');

  assert.match(operations, /PageToolbar/);
  assert.match(operations, /Dialog/);
  assert.match(operations, /DialogContent/);
  assert.match(operations, /Targeted reload/);
  assert.match(operations, /Latest reload report/);
});

test('admin pages remove top section heroes so real workspace content starts immediately', () => {
  const overview = read('packages/sdkwork-router-admin-overview/src/index.tsx');
  const users = read('packages/sdkwork-router-admin-users/src/index.tsx');
  const tenants = read('packages/sdkwork-router-admin-tenants/src/index.tsx');
  const coupons = read('packages/sdkwork-router-admin-coupons/src/index.tsx');
  const catalog = read('packages/sdkwork-router-admin-catalog/src/index.tsx');
  const traffic = read('packages/sdkwork-router-admin-traffic/src/index.tsx');
  const operations = read('packages/sdkwork-router-admin-operations/src/index.tsx');

  assert.doesNotMatch(overview, /SectionHero/);
  assert.doesNotMatch(users, /SectionHero/);
  assert.doesNotMatch(tenants, /SectionHero/);
  assert.doesNotMatch(coupons, /SectionHero/);
  assert.doesNotMatch(catalog, /SectionHero/);
  assert.doesNotMatch(traffic, /SectionHero/);
  assert.doesNotMatch(operations, /SectionHero/);
  assert.match(overview, /adminx-stat-grid/);
  assert.doesNotMatch(overview, /Data-source posture/);
});

test('settings center copy frames shell continuity instead of a standalone preferences page', () => {
  const settings = read('packages/sdkwork-router-admin-settings/src/Settings.tsx');
  const workspace = read('packages/sdkwork-router-admin-settings/src/WorkspaceSettings.tsx');

  assert.match(settings, /control plane|settings center|workspace/i);
  assert.match(workspace, /right canvas|content region|shell posture/i);
});
