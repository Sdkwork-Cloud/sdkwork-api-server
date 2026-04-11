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

test('overview uses shared management workbench and shared stat primitives', () => {
  const overview = read('packages/sdkwork-router-admin-overview/src/index.tsx');
  const packageJson = readJson('packages/sdkwork-router-admin-overview/package.json');

  assert.match(overview, /@sdkwork\/ui-pc-react/);
  assert.match(overview, /ManagementWorkbench/);
  assert.match(overview, /StatCard/);
  assert.doesNotMatch(overview, /adminx-/);
  assert.doesNotMatch(overview, /sdkwork-router-admin-commons/);
  assert.equal(packageJson.dependencies['@sdkwork/ui-pc-react'], sharedUiPackagePath);
  assert.equal(packageJson.dependencies['sdkwork-router-admin-commons'], undefined);
});

test('users, tenants, coupons, and gateway CRUD pages all use the shared ui package instead of legacy page scaffolding', () => {
  const users = {
    source: read('packages/sdkwork-router-admin-users/src/index.tsx'),
    packageJson: readJson('packages/sdkwork-router-admin-users/package.json'),
  };
  const tenants = {
    source: read('packages/sdkwork-router-admin-tenants/src/index.tsx'),
    packageJson: readJson('packages/sdkwork-router-admin-tenants/package.json'),
  };
  const workbenchCoverage = [
    {
      source: read('packages/sdkwork-router-admin-coupons/src/index.tsx'),
      packageJson: readJson('packages/sdkwork-router-admin-coupons/package.json'),
    },
  ];
  const gatewayWorkbenchCoverage = [
    {
      source: read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx'),
      packageJson: readJson('packages/sdkwork-router-admin-apirouter/package.json'),
    },
    {
      source: read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayRoutesPage.tsx'),
      packageJson: readJson('packages/sdkwork-router-admin-apirouter/package.json'),
    },
    {
      source: read(
        'packages/sdkwork-router-admin-apirouter/src/pages/GatewayModelMappingsPage.tsx',
      ),
      packageJson: readJson('packages/sdkwork-router-admin-apirouter/package.json'),
    },
    {
      source: read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx'),
      packageJson: readJson('packages/sdkwork-router-admin-apirouter/package.json'),
    },
  ];

  assert.match(users.source, /@sdkwork\/ui-pc-react/);
  assert.match(users.source, /Dialog/);
  assert.match(users.source, /Input/);
  assert.match(users.source, /Button/);
  assert.doesNotMatch(users.source, /adminx-/);
  assert.doesNotMatch(users.source, /sdkwork-router-admin-commons/);
  assert.equal(users.packageJson.dependencies['@sdkwork/ui-pc-react'], sharedUiPackagePath);
  assert.equal(users.packageJson.dependencies['sdkwork-router-admin-commons'], undefined);

  assert.match(tenants.source, /@sdkwork\/ui-pc-react/);
  assert.match(tenants.source, /Dialog/);
  assert.match(tenants.source, /Input/);
  assert.match(tenants.source, /Button/);
  assert.doesNotMatch(tenants.source, /adminx-/);
  assert.doesNotMatch(tenants.source, /sdkwork-router-admin-commons/);
  assert.equal(tenants.packageJson.dependencies['@sdkwork/ui-pc-react'], sharedUiPackagePath);
  assert.equal(tenants.packageJson.dependencies['sdkwork-router-admin-commons'], undefined);

  for (const { source, packageJson } of workbenchCoverage) {
    assert.match(source, /@sdkwork\/ui-pc-react/);
    assert.match(source, /Drawer/);
    assert.match(source, /Input/);
    assert.match(source, /Canonical marketing derived/);
    assert.match(source, /Template governance/);
    assert.doesNotMatch(source, /CrudWorkbench|ManagementWorkbench/);
    assert.doesNotMatch(source, /adminx-/);
    assert.doesNotMatch(source, /sdkwork-router-admin-commons/);
    assert.equal(packageJson.dependencies['@sdkwork/ui-pc-react'], sharedUiPackagePath);
    assert.equal(packageJson.dependencies['sdkwork-router-admin-commons'], undefined);
  }

  for (const { source, packageJson } of gatewayWorkbenchCoverage) {
    assert.match(source, /@sdkwork\/ui-pc-react/);
    assert.match(source, /Drawer|Dialog/);
    assert.match(source, /Input/);
    assert.match(source, /Button/);
    assert.doesNotMatch(source, /CrudWorkbench|ManagementWorkbench/);
    assert.doesNotMatch(source, /adminx-/);
    assert.doesNotMatch(source, /sdkwork-router-admin-commons/);
    assert.equal(packageJson.dependencies['@sdkwork/ui-pc-react'], sharedUiPackagePath);
    assert.equal(packageJson.dependencies['sdkwork-router-admin-commons'], undefined);
  }
});

test('users adopts admin-table workflow with a top query form, paginated registry, and drawer-based detail surface', () => {
  const users = read('packages/sdkwork-router-admin-users/src/index.tsx');
  const registry = read('packages/sdkwork-router-admin-users/src/page/UsersRegistrySection.tsx');
  const drawer = read('packages/sdkwork-router-admin-users/src/page/UsersDetailDrawer.tsx');
  const shared = read('packages/sdkwork-router-admin-users/src/page/shared.tsx');

  assert.match(users, /<form/);
  assert.match(users, /Search users/);
  assert.match(users, /New Operator/);
  assert.match(users, /New Portal User/);
  assert.match(users, /flex flex-wrap items-center gap-3/);
  assert.doesNotMatch(users, /flex flex-wrap items-end gap-3/);
  assert.doesNotMatch(users, /UsersManagementWorkbench/);
  assert.doesNotMatch(users, /title="Users"/);
  assert.doesNotMatch(users, /PageHeader/);
  assert.match(shared, /labelVisibility\?: 'visible' \| 'sr-only'/);
  assert.match(shared, /const isHiddenLabel = labelVisibility === 'sr-only';/);
  assert.match(shared, /className=\{isHiddenLabel \? 'space-y-0' : 'space-y-2'\}/);
  assert.match(shared, /<Label className=\{isHiddenLabel \? 'sr-only' : undefined\}/);
  assert.match(drawer, /Drawer/);
  assert.match(drawer, /DrawerContent/);
  assert.match(drawer, /DrawerHeader/);
  assert.match(drawer, /DrawerBody/);
  assert.match(registry, /DataTable/);
  assert.match(registry, /Pagination/);
  assert.match(registry, /stickyHeader/);
});

test('tenants adopts admin-table workflow with a top query form, paginated registry, and drawer-based detail surface', () => {
  const tenants = read('packages/sdkwork-router-admin-tenants/src/index.tsx');
  const registry = read('packages/sdkwork-router-admin-tenants/src/page/TenantsRegistrySection.tsx');
  const drawer = read('packages/sdkwork-router-admin-tenants/src/page/TenantsDetailDrawer.tsx');

  assert.match(tenants, /<form/);
  assert.match(tenants, /Search tenants/);
  assert.match(tenants, /New tenant/);
  assert.match(tenants, /New project/);
  assert.match(tenants, /Issue gateway key/);
  assert.doesNotMatch(tenants, /TenantsManagementWorkbench/);
  assert.doesNotMatch(tenants, /FilterBar/);
  assert.doesNotMatch(tenants, /title="Tenants"/);
  assert.doesNotMatch(tenants, /PageHeader/);
  assert.match(drawer, /Drawer/);
  assert.match(drawer, /DrawerContent/);
  assert.match(drawer, /DrawerHeader/);
  assert.match(drawer, /DrawerBody/);
  assert.match(registry, /DataTable/);
  assert.match(registry, /Pagination/);
  assert.match(registry, /stickyHeader/);
});

test('coupons adopts admin-table workflow with a top query form, paginated registry, and drawer-based detail surface', () => {
  const coupons = read('packages/sdkwork-router-admin-coupons/src/index.tsx');
  const registry = read(
    'packages/sdkwork-router-admin-coupons/src/page/CouponsRegistrySection.tsx',
  );
  const drawer = read(
    'packages/sdkwork-router-admin-coupons/src/page/CouponsDetailDrawer.tsx',
  );

  assert.match(coupons, /<form/);
  assert.match(coupons, /Search campaigns/);
  assert.match(coupons, /Canonical marketing derived/);
  assert.match(coupons, /Template governance/);
  assert.doesNotMatch(coupons, /New coupon/);
  assert.doesNotMatch(coupons, /CouponsManagementWorkbench/);
  assert.doesNotMatch(coupons, /FilterBar/);
  assert.doesNotMatch(coupons, /title="Coupons"/);
  assert.doesNotMatch(coupons, /PageHeader/);
  assert.match(drawer, /Drawer/);
  assert.match(drawer, /DrawerContent/);
  assert.match(drawer, /DrawerHeader/);
  assert.match(drawer, /DrawerBody/);
  assert.match(registry, /DataTable/);
  assert.match(registry, /Pagination/);
  assert.match(registry, /stickyHeader/);
});

test('catalog adopts admin-table workflow with a top query form, paginated registry, and drawer-based detail surface', () => {
  const catalog = read('packages/sdkwork-router-admin-catalog/src/index.tsx');
  const registry = read(
    'packages/sdkwork-router-admin-catalog/src/page/CatalogRegistrySection.tsx',
  );
  const drawer = read(
    'packages/sdkwork-router-admin-catalog/src/page/CatalogDetailDrawer.tsx',
  );

  assert.match(catalog, /<form/);
  assert.match(catalog, /Search catalog/);
  assert.match(catalog, /Catalog area/);
  assert.match(catalog, /CatalogDetailDrawer/);
  assert.doesNotMatch(catalog, /CatalogManagementWorkbench/);
  assert.doesNotMatch(catalog, /ManagementWorkbench/);
  assert.doesNotMatch(catalog, /FilterBar/);
  assert.doesNotMatch(catalog, /title="Catalog"/);
  assert.match(drawer, /Drawer/);
  assert.match(drawer, /DrawerContent/);
  assert.match(drawer, /DrawerHeader/);
  assert.match(drawer, /DrawerBody/);
  assert.match(registry, /DataTable/);
  assert.match(registry, /Pagination/);
  assert.match(registry, /stickyHeader/);
});

test('gateway access adopts admin-table workflow with a top query form, paginated registry, and drawer-based detail surface', () => {
  const page = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx');
  const registry = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessRegistrySection.tsx',
  );
  const drawer = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessDetailDrawer.tsx',
  );
  const createDialog = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayApiKeyCreateDialog.tsx',
  );
  const editDialog = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayApiKeyEditDialog.tsx',
  );
  const groupsDialog = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayApiKeyGroupsDialog.tsx',
  );

  assert.match(page, /<form/);
  assert.match(page, /Search API keys/);
  assert.match(page, /Create API key/);
  assert.match(page, /Manage groups/);
  assert.doesNotMatch(page, /GatewayManagementWorkbench/);
  assert.doesNotMatch(page, /FilterBar/);
  assert.doesNotMatch(page, /title="Gateway access"/);
  assert.match(createDialog, /API key group/);
  assert.match(editDialog, /API key group/);
  assert.match(groupsDialog, /Search groups/);
  assert.match(groupsDialog, /Default scope/);
  assert.match(groupsDialog, /Accounting mode/);
  assert.match(groupsDialog, /No routing profile override/);
  assert.match(groupsDialog, /routingProfiles\.filter/);
  assert.match(groupsDialog, /profile\.active/);
  assert.match(groupsDialog, /profile\.profile_id === draft\.default_routing_profile_id/);
  assert.match(drawer, /Drawer/);
  assert.match(drawer, /DrawerContent/);
  assert.match(drawer, /DrawerHeader/);
  assert.match(drawer, /DrawerBody/);
  assert.match(registry, /DataTable/);
  assert.match(registry, /Pagination/);
  assert.match(registry, /stickyHeader/);
});

test('gateway routes adopts admin-table workflow with a top query form, paginated registry, and drawer-based detail surface', () => {
  const page = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayRoutesPage.tsx');
  const registry = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutesRegistrySection.tsx',
  );
  const drawer = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutesDetailDrawer.tsx',
  );
  const detailPanel = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutesDetailPanel.tsx',
  );
  const profilesDialog = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutingProfilesDialog.tsx',
  );
  const snapshotDialog = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutingSnapshotsDialog.tsx',
  );

  assert.match(page, /<form/);
  assert.match(page, /Search providers/);
  assert.match(page, /New route provider/);
  assert.match(page, /Manage routing profiles/);
  assert.match(page, /Compiled snapshots/);
  assert.match(page, /Bound groups/);
  assert.match(page, /Snapshot evidence/);
  assert.doesNotMatch(page, /GatewayManagementWorkbench/);
  assert.doesNotMatch(page, /FilterBar/);
  assert.doesNotMatch(page, /title="Gateway routes"/);
  assert.match(profilesDialog, /Routing profiles/);
  assert.match(profilesDialog, /Preferred region/);
  assert.match(profilesDialog, /Require healthy/);
  assert.match(profilesDialog, /Provider order/);
  assert.match(snapshotDialog, /Search compiled snapshots/);
  assert.match(snapshotDialog, /Applied routing profile/);
  assert.match(snapshotDialog, /Route key/);
  assert.match(snapshotDialog, /Capability/);
  assert.match(snapshotDialog, /Provider order/);
  assert.match(detailPanel, /Routing impact/);
  assert.match(detailPanel, /Compiled snapshots/);
  assert.match(detailPanel, /Bound groups/);
  assert.match(detailPanel, /Default provider/);
  assert.match(drawer, /Drawer/);
  assert.match(drawer, /DrawerContent/);
  assert.match(drawer, /DrawerHeader/);
  assert.match(drawer, /DrawerBody/);
  assert.match(registry, /DataTable/);
  assert.match(registry, /Pagination/);
  assert.match(registry, /stickyHeader/);
});

test('gateway model mappings adopts admin-table workflow with a top query form, paginated registry, and drawer-based detail surface', () => {
  const page = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayModelMappingsPage.tsx',
  );
  const registry = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/mappings/GatewayModelMappingsRegistrySection.tsx',
  );
  const drawer = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/mappings/GatewayModelMappingsDetailDrawer.tsx',
  );

  assert.match(page, /<form/);
  assert.match(page, /Search mappings/);
  assert.match(page, /New model mapping/);
  assert.doesNotMatch(page, /GatewayManagementWorkbench/);
  assert.doesNotMatch(page, /FilterBar/);
  assert.doesNotMatch(page, /title="Gateway model mappings"/);
  assert.match(drawer, /Drawer/);
  assert.match(drawer, /DrawerContent/);
  assert.match(drawer, /DrawerHeader/);
  assert.match(drawer, /DrawerBody/);
  assert.match(registry, /DataTable/);
  assert.match(registry, /Pagination/);
  assert.match(registry, /stickyHeader/);
});

test('gateway usage adopts admin-table workflow with a top query form, paginated registry, and drawer-based detail surface', () => {
  const page = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx');
  const registry = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/usage/GatewayUsageRegistrySection.tsx',
  );
  const drawer = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/usage/GatewayUsageDetailDrawer.tsx',
  );

  assert.match(page, /<form/);
  assert.match(page, /Search usage/);
  assert.match(page, /Export usage CSV/);
  assert.match(page, /Billing events/);
  assert.match(page, /Group chargeback/);
  assert.match(page, /Capability mix/);
  assert.match(page, /Accounting mode/);
  assert.doesNotMatch(page, /GatewayManagementWorkbench/);
  assert.doesNotMatch(page, /FilterBar/);
  assert.doesNotMatch(page, /title="Gateway usage"/);
  assert.match(drawer, /Drawer/);
  assert.match(drawer, /DrawerContent/);
  assert.match(drawer, /DrawerHeader/);
  assert.match(drawer, /DrawerBody/);
  assert.match(registry, /DataTable/);
  assert.match(registry, /Pagination/);
  assert.match(registry, /stickyHeader/);
});

test('admin theme tightens shared select dropdown corners and keeps long values single-line', () => {
  const theme = read('src/theme.css');

  assert.match(theme, /--admin-select-content-radius:/);
  assert.match(theme, /\[data-sdk-ui="select-content"\]/);
  assert.match(theme, /border-radius:\s*var\(--admin-select-content-radius\)/);
  assert.match(theme, /max-width:\s*min\(32rem, calc\(100vw - 2rem\)\)/);
  assert.match(theme, /\[data-sdk-ui="select-trigger"\]\s*>\s*span/);
  assert.match(theme, /\[data-sdk-ui="select-item"\]\s*>\s*:last-child/);
  assert.match(theme, /white-space:\s*nowrap/);
  assert.match(theme, /text-overflow:\s*ellipsis/);
  assert.match(theme, /overflow:\s*hidden/);
});
