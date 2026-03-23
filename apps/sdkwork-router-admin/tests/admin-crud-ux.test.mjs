import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('admin commons exposes toolbar, dialog, and field primitives for focused CRUD flows', () => {
  const commons = read('packages/sdkwork-router-admin-commons/src/index.tsx');

  assert.match(commons, /PageToolbar/);
  assert.match(commons, /Dialog/);
  assert.match(commons, /DialogContent/);
  assert.match(commons, /DialogFooter/);
  assert.match(commons, /FormField/);
});

test('users page moves create and edit flows into focused dialogs', () => {
  const users = read('packages/sdkwork-router-admin-users/src/index.tsx');

  assert.match(users, /PageToolbar/);
  assert.match(users, /Dialog/);
  assert.match(users, /DialogContent/);
  assert.match(users, /New operator/);
  assert.match(users, /New portal user/);
  assert.match(users, /Edit operator/);
  assert.match(users, /Edit portal user/);
  assert.match(users, /ConfirmDialog/);
});

test('tenants page keeps registries primary and opens tenant, project, and key workflows in dialogs', () => {
  const tenants = read('packages/sdkwork-router-admin-tenants/src/index.tsx');

  assert.match(tenants, /PageToolbar/);
  assert.match(tenants, /Dialog/);
  assert.match(tenants, /DialogContent/);
  assert.match(tenants, /New tenant/);
  assert.match(tenants, /New project/);
  assert.match(tenants, /Issue gateway key/);
  assert.match(tenants, /Key label/);
  assert.match(tenants, /Notes/);
  assert.match(tenants, /Expires at \(ms\)/);
  assert.match(tenants, /Plaintext key ready/);
});

test('coupons page turns campaign editing into modal-driven workflow', () => {
  const coupons = read('packages/sdkwork-router-admin-coupons/src/index.tsx');

  assert.match(coupons, /PageToolbar/);
  assert.match(coupons, /Dialog/);
  assert.match(coupons, /DialogContent/);
  assert.match(coupons, /New coupon/);
  assert.match(coupons, /Edit coupon campaign/);
  assert.match(coupons, /ConfirmDialog/);
});

test('catalog page moves channel, provider, credential, model, and pricing maintenance out of the registry canvas', () => {
  const catalog = read('packages/sdkwork-router-admin-catalog/src/index.tsx');
  const types = read('packages/sdkwork-router-admin-types/src/index.ts');

  assert.match(catalog, /PageToolbar/);
  assert.match(catalog, /Dialog/);
  assert.match(catalog, /DialogContent/);
  assert.match(catalog, /ConfirmDialog/);
  assert.match(catalog, /New channel/);
  assert.match(catalog, /New proxy provider/);
  assert.match(catalog, /Rotate credential/);
  assert.match(catalog, /Manage models/);
  assert.match(catalog, /New channel model/);
  assert.match(catalog, /Manage pricing/);
  assert.match(catalog, /New model pricing/);
  assert.match(catalog, /price_unit: 'per_1m_tokens'/);
  assert.match(types, /notes\?: string \| null;/);
  assert.doesNotMatch(catalog, /Channel maintenance/);
  assert.doesNotMatch(catalog, /Credential maintenance/);
});

test('operations page uses a targeted reload dialog instead of leaving a persistent inline form in the workspace', () => {
  const operations = read('packages/sdkwork-router-admin-operations/src/index.tsx');

  assert.match(operations, /PageToolbar/);
  assert.match(operations, /Dialog/);
  assert.match(operations, /DialogContent/);
  assert.match(operations, /Targeted reload/);
  assert.match(operations, /Latest reload report/);
  assert.doesNotMatch(operations, /title="Reload runtimes"/);
});
