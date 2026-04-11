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

test('traffic and operations use paginated admin-table patterns instead of legacy admin workbenches', () => {
  const coverage = [
    {
      packageJson: readJson('packages/sdkwork-router-admin-traffic/package.json'),
      source: read('packages/sdkwork-router-admin-traffic/src/index.tsx'),
    },
    {
      packageJson: readJson('packages/sdkwork-router-admin-operations/package.json'),
      source: read('packages/sdkwork-router-admin-operations/src/index.tsx'),
    },
  ];

  for (const { packageJson, source } of coverage) {
    assert.match(source, /@sdkwork\/ui-pc-react/);
    assert.match(source, /<form/);
    assert.match(source, /DataTable/);
    assert.match(source, /Pagination/);
    assert.match(source, /Drawer|Dialog/);
    assert.match(source, /stickyHeader/);
    assert.doesNotMatch(source, /ManagementWorkbench|CrudWorkbench/);
    assert.doesNotMatch(source, /FilterBar/);
    assert.doesNotMatch(source, /adminx-/);
    assert.doesNotMatch(source, /sdkwork-router-admin-commons/);
    assert.equal(
      packageJson.dependencies['@sdkwork/ui-pc-react'],
      sharedUiPackagePath,
    );
    assert.equal(packageJson.dependencies['sdkwork-router-admin-commons'], undefined);
  }
});

test('users and tenants tighten detail metrics and flatten embedded registry shells', () => {
  const registry = read('packages/sdkwork-router-admin-users/src/page/UsersRegistrySection.tsx');
  const usersDetailPanel = read('packages/sdkwork-router-admin-users/src/page/UsersDetailPanel.tsx');
  const tenantsDetailPanel = read('packages/sdkwork-router-admin-tenants/src/page/TenantsDetailPanel.tsx');

  assert.match(registry, /embeddedAdminDataTableSlotProps/);
  assert.match(usersDetailPanel, /rounded-\[var\(--sdk-radius-control\)\]/);
  assert.doesNotMatch(usersDetailPanel, /rounded-2xl/);
  assert.match(tenantsDetailPanel, /rounded-\[var\(--sdk-radius-control\)\]/);
  assert.doesNotMatch(tenantsDetailPanel, /rounded-2xl/);
});

test('admin registry tables share a single embedded DataTable shell contract', () => {
  const coreIndex = read('packages/sdkwork-router-admin-core/src/index.tsx');
  const tableShell = read('packages/sdkwork-router-admin-core/src/tableShell.ts');
  const coverage = [
    read('packages/sdkwork-router-admin-users/src/page/UsersRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-tenants/src/page/TenantsRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-coupons/src/page/CouponsRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-catalog/src/page/CatalogRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutesRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/mappings/GatewayModelMappingsRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/usage/GatewayUsageRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-traffic/src/index.tsx'),
    read('packages/sdkwork-router-admin-operations/src/index.tsx'),
  ];

  assert.match(coreIndex, /embeddedAdminDataTableClassName/);
  assert.match(coreIndex, /embeddedAdminDataTableSlotProps/);
  assert.match(tableShell, /embeddedAdminDataTableClassName/);
  assert.match(tableShell, /embeddedAdminDataTableSlotProps/);
  assert.match(tableShell, /rounded-none border-0 bg-transparent shadow-none/);

  for (const source of coverage) {
    assert.match(source, /embeddedAdminDataTableClassName/);
    assert.match(source, /embeddedAdminDataTableSlotProps/);
  }
});

test('admin tables remove top summary strips so forms lead directly into table headers', () => {
  const registryCoverage = [
    read('packages/sdkwork-router-admin-users/src/page/UsersRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-tenants/src/page/TenantsRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-coupons/src/page/CouponsRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-catalog/src/page/CatalogRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutesRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/mappings/GatewayModelMappingsRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/usage/GatewayUsageRegistrySection.tsx'),
  ];
  const traffic = read('packages/sdkwork-router-admin-traffic/src/index.tsx');
  const operations = read('packages/sdkwork-router-admin-operations/src/index.tsx');

  for (const source of registryCoverage) {
    assert.doesNotMatch(
      source,
      /<Card className="h-full flex flex-col overflow-hidden p-0">\s*<div className="flex flex-wrap items-center justify-between gap-3 border-b/s,
    );
  }

  assert.doesNotMatch(
    traffic,
    /<Card className="min-h-0 flex-1 flex flex-col overflow-hidden p-0">\s*<div className="flex flex-wrap items-center justify-between gap-3 border-b/s,
  );
  assert.doesNotMatch(
    operations,
    /<Card className="min-h-0 flex-1 flex flex-col overflow-hidden p-0">\s*<div className="flex flex-wrap items-center justify-between gap-3 border-b/s,
  );
  assert.doesNotMatch(traffic, /description=\{tableCopy\.description\}/);
  assert.doesNotMatch(traffic, /title=\{tableCopy\.title\}/);
  assert.doesNotMatch(operations, /description=\{tableCopy\.description\}/);
  assert.doesNotMatch(operations, /title=\{tableCopy\.title\}/);
});

test('single-select admin tables use row highlight props instead of the shared bulk selection bar', () => {
  const coreIndex = read('packages/sdkwork-router-admin-core/src/index.tsx');
  const tableShell = read('packages/sdkwork-router-admin-core/src/tableShell.ts');
  const coverage = [
    read('packages/sdkwork-router-admin-users/src/page/UsersRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-tenants/src/page/TenantsRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-coupons/src/page/CouponsRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-catalog/src/page/CatalogRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutesRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/mappings/GatewayModelMappingsRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-apirouter/src/pages/usage/GatewayUsageRegistrySection.tsx'),
    read('packages/sdkwork-router-admin-traffic/src/index.tsx'),
    read('packages/sdkwork-router-admin-operations/src/index.tsx'),
  ];

  assert.match(coreIndex, /buildEmbeddedAdminSingleSelectRowProps/);
  assert.match(tableShell, /buildEmbeddedAdminSingleSelectRowProps/);
  assert.match(tableShell, /bg-\[var\(--sdk-color-brand-primary-soft\)\]/);

  for (const source of coverage) {
    assert.match(source, /buildEmbeddedAdminSingleSelectRowProps/);
    assert.doesNotMatch(source, /selectedRowIds=\{/);
  }
});
