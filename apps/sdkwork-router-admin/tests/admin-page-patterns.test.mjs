import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('protected routes use shared loading and host route surfaces instead of legacy page frame wrappers', () => {
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');

  assert.match(routes, /LoadingBlock/);
  assert.match(routes, /admin-shell-route-stage/);
  assert.match(routes, /admin-shell-route-scroll/);
  assert.doesNotMatch(routes, /PageFrame/);
  assert.doesNotMatch(routes, /adminx-page-frame/);
  assert.doesNotMatch(routes, /adminx-shell-loading/);
});

test('shell host stylesheet is reduced to host layout rules and contains no legacy adminx selectors', () => {
  const styles = read('packages/sdkwork-router-admin-shell/src/styles/shell-host.css');

  assert.match(styles, /\.admin-shell-route-scroll/);
  assert.match(styles, /\[data-sdk-shell='router-admin-desktop']/);
  assert.doesNotMatch(styles, /\.admin-shell-auth-stage/);
  assert.doesNotMatch(styles, /adminx-/);
});

test('users, tenants, coupons, and gateway pages delegate registry, detail, and dialog surfaces to local modules', () => {
  const users = read('packages/sdkwork-router-admin-users/src/index.tsx');
  const usersDrawer = read('packages/sdkwork-router-admin-users/src/page/UsersDetailDrawer.tsx');
  const tenants = read('packages/sdkwork-router-admin-tenants/src/index.tsx');
  const tenantsDrawer = read('packages/sdkwork-router-admin-tenants/src/page/TenantsDetailDrawer.tsx');
  const coupons = read('packages/sdkwork-router-admin-coupons/src/index.tsx');
  const couponsDrawer = read(
    'packages/sdkwork-router-admin-coupons/src/page/CouponsDetailDrawer.tsx',
  );
  const catalog = read('packages/sdkwork-router-admin-catalog/src/index.tsx');
  const catalogDrawer = read(
    'packages/sdkwork-router-admin-catalog/src/page/CatalogDetailDrawer.tsx',
  );
  const catalogDialogs = read(
    'packages/sdkwork-router-admin-catalog/src/page/CatalogDialogs.tsx',
  );
  const gatewayAccess = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx',
  );
  const gatewayAccessDrawer = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessDetailDrawer.tsx',
  );
  const gatewayRoutes = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayRoutesPage.tsx',
  );
  const gatewayRoutesDrawer = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutesDetailDrawer.tsx',
  );
  const gatewayMappings = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayModelMappingsPage.tsx',
  );
  const gatewayMappingsDrawer = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/mappings/GatewayModelMappingsDetailDrawer.tsx',
  );
  const gatewayUsage = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx',
  );
  const gatewayUsageDrawer = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/usage/GatewayUsageDetailDrawer.tsx',
  );

  assert.match(users, /UsersDetailDrawer/);
  assert.match(users, /UsersRegistrySection/);
  assert.match(users, /OperatorUserDialog/);
  assert.match(users, /PortalUserDialog/);
  assert.match(users, /<form/);
  assert.doesNotMatch(users, /UsersManagementWorkbench/);
  assert.doesNotMatch(users, /detail=\{/);
  assert.doesNotMatch(users, /title="Users"/);
  assert.doesNotMatch(users, /DialogContent/);
  assert.match(usersDrawer, /UsersDetailPanel/);
  assert.match(usersDrawer, /Drawer/);
  assert.match(usersDrawer, /DrawerContent/);
  assert.match(usersDrawer, /DrawerHeader/);
  assert.match(usersDrawer, /DrawerBody/);

  assert.match(tenants, /TenantsDetailDrawer/);
  assert.match(tenants, /TenantsRegistrySection/);
  assert.match(tenants, /TenantDialog/);
  assert.match(tenants, /ProjectDialog/);
  assert.match(tenants, /ApiKeyDialog/);
  assert.match(tenants, /PlaintextApiKeyDialog/);
  assert.match(tenants, /<form/);
  assert.doesNotMatch(tenants, /TenantsManagementWorkbench/);
  assert.doesNotMatch(tenants, /detail=\{/);
  assert.doesNotMatch(tenants, /title="Tenants"/);
  assert.doesNotMatch(tenants, /DialogContent/);
  assert.doesNotMatch(tenants, /Textarea/);
  assert.match(tenantsDrawer, /TenantsDetailPanel/);
  assert.match(tenantsDrawer, /Drawer/);
  assert.match(tenantsDrawer, /DrawerContent/);
  assert.match(tenantsDrawer, /DrawerHeader/);
  assert.match(tenantsDrawer, /DrawerBody/);

  assert.match(coupons, /CouponsDetailDrawer/);
  assert.match(coupons, /CouponsRegistrySection/);
  assert.match(coupons, /Canonical marketing derived/);
  assert.match(coupons, /Template governance/);
  assert.match(coupons, /<form/);
  assert.doesNotMatch(coupons, /CouponsManagementWorkbench/);
  assert.doesNotMatch(coupons, /CouponDialog/);
  assert.doesNotMatch(coupons, /detail=\{/);
  assert.doesNotMatch(coupons, /title="Coupons"/);
  assert.doesNotMatch(coupons, /DialogContent/);
  assert.doesNotMatch(coupons, /Textarea/);
  assert.match(couponsDrawer, /CouponsDetailPanel/);
  assert.match(couponsDrawer, /Drawer/);
  assert.match(couponsDrawer, /DrawerContent/);
  assert.match(couponsDrawer, /DrawerHeader/);
  assert.match(couponsDrawer, /DrawerBody/);

  assert.match(catalog, /CatalogDetailDrawer/);
  assert.match(catalog, /CatalogRegistrySection/);
  assert.match(catalog, /CatalogChannelDialog/);
  assert.match(catalog, /CatalogProviderDialog/);
  assert.match(catalog, /CatalogCredentialDialog/);
  assert.match(catalog, /CatalogChannelModelDialog/);
  assert.match(catalog, /CatalogModelPriceDialog/);
  assert.match(catalog, /useCatalogWorkspaceState/);
  assert.match(catalog, /<form/);
  assert.doesNotMatch(catalog, /CatalogManagementWorkbench/);
  assert.doesNotMatch(catalog, /ManagementWorkbench/);
  assert.doesNotMatch(catalog, /FilterBar/);
  assert.doesNotMatch(catalog, /DialogContent/);
  assert.doesNotMatch(catalog, /DataTable/);
  assert.doesNotMatch(catalog, /useState\(/);
  assert.doesNotMatch(catalog, /useEffect\(/);
  assert.doesNotMatch(catalog, /useMemo\(/);
  assert.doesNotMatch(catalog, /useDeferredValue\(/);
  assert.match(catalogDrawer, /CatalogDetailPanel/);
  assert.match(catalogDrawer, /Drawer/);
  assert.match(catalogDrawer, /DrawerContent/);
  assert.match(catalogDrawer, /DrawerHeader/);
  assert.match(catalogDrawer, /DrawerBody/);
  assert.match(
    catalogDialogs,
    /export \{ CatalogChannelDialog \} from '\.\/CatalogChannelDialog';/,
  );
  assert.match(
    catalogDialogs,
    /export \{ CatalogProviderDialog \} from '\.\/CatalogProviderDialog';/,
  );
  assert.match(
    catalogDialogs,
    /export \{ CatalogCredentialDialog \} from '\.\/CatalogCredentialDialog';/,
  );
  assert.match(
    catalogDialogs,
    /export \{ CatalogChannelModelDialog \} from '\.\/CatalogChannelModelDialog';/,
  );
  assert.match(
    catalogDialogs,
    /export \{ CatalogModelPriceDialog \} from '\.\/CatalogModelPriceDialog';/,
  );
  assert.doesNotMatch(catalogDialogs, /DialogContent/);
  assert.doesNotMatch(catalogDialogs, /FormSection/);

  assert.match(gatewayAccess, /GatewayAccessDetailDrawer/);
  assert.match(gatewayAccess, /GatewayAccessRegistrySection/);
  assert.match(gatewayAccess, /GatewayApiKeyCreateDialog/);
  assert.match(gatewayAccess, /GatewayApiKeyEditDialog/);
  assert.match(gatewayAccess, /GatewayApiKeyRouteDialog/);
  assert.match(gatewayAccess, /GatewayApiKeyUsageDialog/);
  assert.match(gatewayAccess, /useGatewayAccessWorkspaceState/);
  assert.match(gatewayAccess, /<form/);
  assert.doesNotMatch(gatewayAccess, /GatewayManagementWorkbench/);
  assert.doesNotMatch(gatewayAccess, /detail=\{/);
  assert.doesNotMatch(gatewayAccess, /title="Gateway access"/);
  assert.doesNotMatch(gatewayAccess, /FilterBar/);
  assert.doesNotMatch(gatewayAccess, /DialogContent/);
  assert.doesNotMatch(gatewayAccess, /DataTableColumn/);
  assert.doesNotMatch(gatewayAccess, /StatusBadge/);
  assert.doesNotMatch(gatewayAccess, /useState\(/);
  assert.doesNotMatch(gatewayAccess, /useEffect\(/);
  assert.doesNotMatch(gatewayAccess, /useMemo\(/);
  assert.doesNotMatch(gatewayAccess, /useDeferredValue\(/);
  assert.match(gatewayAccessDrawer, /GatewayAccessDetailPanel/);
  assert.match(gatewayAccessDrawer, /Drawer/);
  assert.match(gatewayAccessDrawer, /DrawerContent/);
  assert.match(gatewayAccessDrawer, /DrawerHeader/);
  assert.match(gatewayAccessDrawer, /DrawerBody/);

  assert.match(gatewayRoutes, /GatewayRoutesDetailDrawer/);
  assert.match(gatewayRoutes, /GatewayRoutesRegistrySection/);
  assert.match(gatewayRoutes, /GatewayProviderDialog/);
  assert.match(gatewayRoutes, /<form/);
  assert.doesNotMatch(gatewayRoutes, /GatewayManagementWorkbench/);
  assert.doesNotMatch(gatewayRoutes, /detail=\{/);
  assert.doesNotMatch(gatewayRoutes, /title="Gateway routes"/);
  assert.doesNotMatch(gatewayRoutes, /FilterBar/);
  assert.match(gatewayRoutesDrawer, /GatewayRoutesDetailPanel/);
  assert.match(gatewayRoutesDrawer, /Drawer/);
  assert.match(gatewayRoutesDrawer, /DrawerContent/);
  assert.match(gatewayRoutesDrawer, /DrawerHeader/);
  assert.match(gatewayRoutesDrawer, /DrawerBody/);

  assert.match(gatewayMappings, /GatewayModelMappingsDetailDrawer/);
  assert.match(gatewayMappings, /GatewayModelMappingsRegistrySection/);
  assert.match(gatewayMappings, /GatewayModelMappingEditorDialog/);
  assert.match(gatewayMappings, /<form/);
  assert.doesNotMatch(gatewayMappings, /GatewayManagementWorkbench/);
  assert.doesNotMatch(gatewayMappings, /detail=\{/);
  assert.doesNotMatch(gatewayMappings, /title="Gateway model mappings"/);
  assert.doesNotMatch(gatewayMappings, /FilterBar/);
  assert.match(gatewayMappingsDrawer, /GatewayModelMappingsDetailPanel/);
  assert.match(gatewayMappingsDrawer, /Drawer/);
  assert.match(gatewayMappingsDrawer, /DrawerContent/);
  assert.match(gatewayMappingsDrawer, /DrawerHeader/);
  assert.match(gatewayMappingsDrawer, /DrawerBody/);

  assert.match(gatewayUsage, /GatewayUsageDetailDrawer/);
  assert.match(gatewayUsage, /GatewayUsageRegistrySection/);
  assert.match(gatewayUsage, /<form/);
  assert.doesNotMatch(gatewayUsage, /GatewayManagementWorkbench/);
  assert.doesNotMatch(gatewayUsage, /detail=\{/);
  assert.doesNotMatch(gatewayUsage, /title="Gateway usage"/);
  assert.doesNotMatch(gatewayUsage, /FilterBar/);
  assert.match(gatewayUsageDrawer, /GatewayUsageDetailPanel/);
  assert.match(gatewayUsageDrawer, /Drawer/);
  assert.match(gatewayUsageDrawer, /DrawerContent/);
  assert.match(gatewayUsageDrawer, /DrawerHeader/);
  assert.match(gatewayUsageDrawer, /DrawerBody/);
});
