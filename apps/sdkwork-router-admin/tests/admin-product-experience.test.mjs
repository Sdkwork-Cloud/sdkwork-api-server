import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function readJson(relativePath) {
  return JSON.parse(read(relativePath));
}

const sharedUiPackagePath = 'workspace:*';

test('auth uses sdkwork shared form primitives instead of legacy admin auth classes', () => {
  const auth = read('packages/sdkwork-router-admin-auth/src/index.tsx');
  const packageJson = readJson('packages/sdkwork-router-admin-auth/package.json');

  assert.match(auth, /@sdkwork\/ui-pc-react/);
  assert.match(auth, /Button/);
  assert.match(auth, /Input/);
  assert.match(auth, /Label/);
  assert.match(auth, /Badge/);
  assert.match(auth, /min-h-screen/);
  assert.match(auth, /items-center/);
  assert.match(auth, /justify-center/);
  assert.match(auth, /bg-zinc-50/);
  assert.match(auth, /dark:bg-zinc-950/);
  assert.match(auth, /max-w-4xl/);
  assert.match(auth, /md:flex-row/);
  assert.match(auth, /bg-zinc-900/);
  assert.match(auth, /QR login/);
  assert.match(auth, /Open app to scan/);
  assert.doesNotMatch(auth, /adminx-auth-/);
  assert.doesNotMatch(auth, /LeadingIconInput/);
  assert.doesNotMatch(auth, /InlineAlert/);
  assert.doesNotMatch(auth, /Shared UI contract/);
  assert.doesNotMatch(auth, /One control plane for operators, routes, and runtime posture/);
  assert.equal(packageJson.dependencies['@sdkwork/ui-pc-react'], sharedUiPackagePath);
});

test('settings center uses sdkwork settings pattern and removes local motion scaffolding', () => {
  const settingsPage = read('packages/sdkwork-router-admin-settings/src/Settings.tsx');
  const generalSettings = read('packages/sdkwork-router-admin-settings/src/GeneralSettings.tsx');
  const appearanceSettings = read('packages/sdkwork-router-admin-settings/src/AppearanceSettings.tsx');
  const navigationSettings = read('packages/sdkwork-router-admin-settings/src/NavigationSettings.tsx');
  const workspaceSettings = read('packages/sdkwork-router-admin-settings/src/WorkspaceSettings.tsx');
  const shared = read('packages/sdkwork-router-admin-settings/src/Shared.tsx');
  const packageJson = readJson('packages/sdkwork-router-admin-settings/package.json');

  assert.match(settingsPage, /SettingsCenter/);
  assert.match(settingsPage, /@sdkwork\/ui-pc-react/);
  assert.doesNotMatch(settingsPage, /motion\/react/);
  assert.doesNotMatch(settingsPage, /SearchInput/);
  assert.match(generalSettings, /@sdkwork\/ui-pc-react/);
  assert.match(generalSettings, /Card|FormSection|FormGrid|SettingsSection/);
  assert.match(appearanceSettings, /@sdkwork\/ui-pc-react/);
  assert.match(appearanceSettings, /Button|Card|Switch|Select/);
  assert.match(navigationSettings, /@sdkwork\/ui-pc-react/);
  assert.match(navigationSettings, /Checkbox|Switch|Card/);
  assert.match(workspaceSettings, /@sdkwork\/ui-pc-react/);
  assert.match(workspaceSettings, /InlineAlert|Card|Badge/);
  assert.match(shared, /@sdkwork\/ui-pc-react/);
  assert.doesNotMatch(
    [
      settingsPage,
      generalSettings,
      appearanceSettings,
      navigationSettings,
      workspaceSettings,
      shared,
    ].join('\n'),
    /adminx-/,
  );
  assert.equal(packageJson.dependencies['@sdkwork/ui-pc-react'], sharedUiPackagePath);
  assert.equal(packageJson.dependencies.motion, undefined);
});

test('catalog and gateway CRUD pages use the modern admin-table shell', () => {
  const catalog = read('packages/sdkwork-router-admin-catalog/src/index.tsx');
  const gatewayAccess = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx');
  const gatewayRoutes = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayRoutesPage.tsx');
  const gatewayModelMappings = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayModelMappingsPage.tsx');
  const gatewayUsage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx');
  const catalogPackageJson = readJson('packages/sdkwork-router-admin-catalog/package.json');
  const apiRouterPackageJson = readJson('packages/sdkwork-router-admin-apirouter/package.json');

  assert.match(catalog, /@sdkwork\/ui-pc-react/);
  assert.match(catalog, /<form/);
  assert.match(catalog, /Drawer/);
  assert.doesNotMatch(catalog, /adminx-/);
  assert.doesNotMatch(catalog, /ManagementWorkbench|CrudWorkbench/);

  for (const page of [
    gatewayAccess,
    gatewayRoutes,
    gatewayModelMappings,
    gatewayUsage,
  ]) {
    assert.match(page, /@sdkwork\/ui-pc-react/);
    assert.match(page, /<form/);
    assert.match(page, /Drawer/);
    assert.doesNotMatch(page, /ManagementWorkbench|CrudWorkbench/);
    assert.doesNotMatch(page, /adminx-/);
  }

  assert.equal(
    catalogPackageJson.dependencies['@sdkwork/ui-pc-react'],
    sharedUiPackagePath,
  );
  assert.equal(
    apiRouterPackageJson.dependencies['@sdkwork/ui-pc-react'],
    sharedUiPackagePath,
  );
});

test('traffic and operations adopt the modern admin-table shell instead of legacy router workbench chrome', () => {
  const traffic = read('packages/sdkwork-router-admin-traffic/src/index.tsx');
  const operations = read('packages/sdkwork-router-admin-operations/src/index.tsx');
  const trafficPackageJson = readJson('packages/sdkwork-router-admin-traffic/package.json');
  const operationsPackageJson = readJson('packages/sdkwork-router-admin-operations/package.json');

  for (const page of [traffic, operations]) {
    assert.match(page, /@sdkwork\/ui-pc-react/);
    assert.match(page, /<form/);
    assert.match(page, /DataTable/);
    assert.match(page, /Pagination/);
    assert.match(page, /Drawer|Dialog/);
    assert.match(page, /InlineAlert|DescriptionList|StatusBadge/);
    assert.doesNotMatch(page, /ManagementWorkbench|CrudWorkbench/);
    assert.doesNotMatch(page, /FilterBar/);
    assert.doesNotMatch(page, /adminx-/);
    assert.doesNotMatch(page, /sdkwork-router-admin-commons/);
  }

  assert.equal(
    trafficPackageJson.dependencies['@sdkwork/ui-pc-react'],
    sharedUiPackagePath,
  );
  assert.equal(trafficPackageJson.dependencies['sdkwork-router-admin-commons'], undefined);
  assert.equal(
    operationsPackageJson.dependencies['@sdkwork/ui-pc-react'],
    sharedUiPackagePath,
  );
  assert.equal(operationsPackageJson.dependencies['sdkwork-router-admin-commons'], undefined);
});
