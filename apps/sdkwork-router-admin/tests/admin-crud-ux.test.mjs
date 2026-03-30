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
  assert.match(operations, /Reload runtimes/);
  assert.doesNotMatch(operations, /title="Reload runtimes"/);
});

test('admin dialog and workbench forms reuse shared Input, Select, and Textarea primitives instead of raw html controls', () => {
  const formHeavyFiles = [
    'packages/sdkwork-router-admin-users/src/index.tsx',
    'packages/sdkwork-router-admin-coupons/src/index.tsx',
    'packages/sdkwork-router-admin-tenants/src/index.tsx',
    'packages/sdkwork-router-admin-operations/src/index.tsx',
    'packages/sdkwork-router-admin-catalog/src/index.tsx',
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayRoutesPage.tsx',
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayModelMappingsPage.tsx',
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx',
  ];

  for (const relativePath of formHeavyFiles) {
    const source = read(relativePath);

    assert.match(source, /sdkwork-router-admin-commons/);
    assert.match(source, /Input/);
    assert.match(source, /Select/);
    assert.doesNotMatch(source, /<input/);
    assert.doesNotMatch(source, /<select/);
    assert.doesNotMatch(source, /<textarea/);
  }
});

test('admin auth, settings, and analytics filters also avoid raw html form controls outside shared primitives', () => {
  const coverage = [
    {
      relativePath: 'packages/sdkwork-router-admin-auth/src/index.tsx',
      requiredTokens: ['Input'],
    },
    {
      relativePath: 'packages/sdkwork-router-admin-traffic/src/index.tsx',
      requiredTokens: ['Select'],
    },
    {
      relativePath: 'packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx',
      requiredTokens: ['Select'],
    },
    {
      relativePath: 'packages/sdkwork-router-admin-settings/src/NavigationSettings.tsx',
      requiredTokens: ['Checkbox'],
    },
  ];

  for (const { relativePath, requiredTokens } of coverage) {
    const source = read(relativePath);

    assert.match(source, /sdkwork-router-admin-commons/);
    for (const token of requiredTokens) {
      assert.match(source, new RegExp(token));
    }
    assert.doesNotMatch(source, /<input/);
    assert.doesNotMatch(source, /<select/);
    assert.doesNotMatch(source, /<textarea/);
  }
});

test('admin compact list toolbars keep search first, actions right, and filters out of the row layout', () => {
  const commons = read('packages/sdkwork-router-admin-commons/src/index.tsx');
  const stylesheet = read('packages/sdkwork-router-admin-shell/src/styles/index.css');
  const listPages = [
    'packages/sdkwork-router-admin-users/src/index.tsx',
    'packages/sdkwork-router-admin-coupons/src/index.tsx',
    'packages/sdkwork-router-admin-traffic/src/index.tsx',
    'packages/sdkwork-router-admin-tenants/src/index.tsx',
    'packages/sdkwork-router-admin-operations/src/index.tsx',
    'packages/sdkwork-router-admin-catalog/src/index.tsx',
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx',
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayRoutesPage.tsx',
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayModelMappingsPage.tsx',
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx',
  ];

  assert.match(commons, /export function ToolbarInline/);
  assert.match(stylesheet, /\.adminx-toolbar-inline\s*\{/);
  assert.match(stylesheet, /flex-wrap:\s*nowrap/);
  assert.match(stylesheet, /\.adminx-toolbar-disclosure-panel[\s\S]*position:\s*absolute/);
  assert.match(
    stylesheet,
    /\.adminx-toolbar-search-input\s+\.adminx-toolbar-search-input-element[\s\S]*padding-left:\s*48px/,
  );

  for (const relativePath of listPages) {
    const source = read(relativePath);

    assert.match(source, /ToolbarInline/);
  }
});
