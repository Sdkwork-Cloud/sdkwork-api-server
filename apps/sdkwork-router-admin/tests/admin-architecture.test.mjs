import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

const requiredPackages = [
  'sdkwork-router-admin-types',
  'sdkwork-router-admin-core',
  'sdkwork-router-admin-shell',
  'sdkwork-router-admin-admin-api',
  'sdkwork-router-admin-apirouter',
  'sdkwork-router-admin-auth',
  'sdkwork-router-admin-overview',
  'sdkwork-router-admin-users',
  'sdkwork-router-admin-tenants',
  'sdkwork-router-admin-coupons',
  'sdkwork-router-admin-catalog',
  'sdkwork-router-admin-traffic',
  'sdkwork-router-admin-operations',
  'sdkwork-router-admin-settings',
];

test('standalone sdkwork-router-admin app root exists', () => {
  assert.equal(existsSync(path.join(appRoot, 'package.json')), true);
  assert.equal(existsSync(path.join(appRoot, 'src', 'App.tsx')), true);
  assert.equal(existsSync(path.join(appRoot, 'src-tauri', 'Cargo.toml')), true);
  assert.equal(existsSync(path.join(appRoot, 'src-tauri', 'src', 'main.rs')), true);
});

test('app root exposes standalone browser and tauri scripts', () => {
  const packageJsonSource = read('package.json');
  const packageJson = JSON.parse(packageJsonSource);

  assert.equal(typeof packageJson.scripts?.dev, 'string');
  assert.equal(typeof packageJson.scripts?.build, 'string');
  assert.equal(typeof packageJson.scripts?.typecheck, 'string');
  assert.equal(typeof packageJson.scripts?.preview, 'string');
  assert.equal(typeof packageJson.scripts?.['tauri:dev'], 'string');
  assert.equal(typeof packageJson.scripts?.['tauri:build'], 'string');
  assert.match(packageJsonSource, /run-vite-cli\.mjs --host 0\.0\.0\.0/);
  assert.match(packageJsonSource, /run-vite-cli\.mjs build/);
  assert.match(packageJsonSource, /run-tsc-cli\.mjs --noEmit/);
  assert.match(packageJsonSource, /run-vite-cli\.mjs preview --host 0\.0\.0\.0 --port 4173 --strictPort/);
});

test('admin typecheck stays on the repo-owned readable TypeScript launcher and local runtime shims', () => {
  const packageJsonSource = read('package.json');
  const tsconfig = read('tsconfig.json');
  const viteEnv = read('src/vite-env.d.ts');

  assert.match(packageJsonSource, /"typecheck": "node \.\.\/\.\.\/scripts\/dev\/run-tsc-cli\.mjs --noEmit"/);
  assert.equal(existsSync(path.join(appRoot, 'src', 'types', 'node-runtime-shim.d.ts')), true);
  assert.equal(existsSync(path.join(appRoot, 'src', 'types', 'vite-client-shim.d.ts')), true);
  assert.equal(existsSync(path.join(appRoot, 'src', 'types', 'sdkwork-ui-pc-react-shim.d.ts')), true);
  assert.doesNotMatch(tsconfig, /"types"\s*:\s*\[\s*"node"\s*,\s*"vite\/client"\s*\]/);
  assert.match(tsconfig, /sdkwork-ui-pc-react-shim\.d\.ts/);
  assert.match(viteEnv, /types\/vite-client-shim\.d\.ts/);
  assert.match(viteEnv, /types\/node-runtime-shim\.d\.ts/);
});

test('required packages exist under packages/', () => {
  for (const packageName of requiredPackages) {
    assert.equal(
      existsSync(path.join(appRoot, 'packages', packageName, 'package.json')),
      true,
      `missing ${packageName}`,
    );
  }
});

test('shell route manifest includes super-admin management sections', () => {
  const routes = read('packages/sdkwork-router-admin-core/src/routes.ts');

  assert.match(routes, /overview/);
  assert.match(routes, /users/);
  assert.match(routes, /tenants/);
  assert.match(routes, /coupons/);
  assert.match(routes, /api-keys/);
  assert.match(routes, /route-config/);
  assert.match(routes, /model-mapping/);
  assert.match(routes, /usage-records/);
  assert.match(routes, /catalog/);
  assert.match(routes, /traffic/);
  assert.match(routes, /operations/);
  assert.match(routes, /settings/);
});

test('admin core exposes a package-first route manifest and shell prefetch derives from it', () => {
  const routeManifest = read('packages/sdkwork-router-admin-core/src/routeManifest.ts');
  const coreIndex = read('packages/sdkwork-router-admin-core/src/index.tsx');
  const prefetch = read(
    'packages/sdkwork-router-admin-shell/src/application/router/routePrefetch.ts',
  );

  assert.match(routeManifest, /adminRouteManifest/);
  assert.match(routeManifest, /moduleId:/);
  assert.match(routeManifest, /sdkwork-router-admin-overview/);
  assert.match(routeManifest, /sdkwork-router-admin-apirouter/);
  assert.match(routeManifest, /resolveAdminPath/);
  assert.match(coreIndex, /adminRouteManifest/);
  assert.match(prefetch, /adminRouteManifest/);
  assert.match(prefetch, /loadAdminRouteModule/);
  assert.doesNotMatch(prefetch, /\[ADMIN_ROUTE_PATHS\.OVERVIEW,\s*\(\)\s*=>\s*import\('sdkwork-router-admin-overview'\)\]/);
});

test('admin core formalizes product module manifests with capability, permission, and lazy-load metadata', () => {
  const types = read('packages/sdkwork-router-admin-types/src/index.ts');
  const routeManifest = read('packages/sdkwork-router-admin-core/src/routeManifest.ts');
  const coreIndex = read('packages/sdkwork-router-admin-core/src/index.tsx');

  assert.match(types, /AdminProductModuleManifest/);
  assert.match(types, /pluginKind: 'admin-module'/);
  assert.match(types, /capabilityTags: string\[]/);
  assert.match(types, /requiredPermissions: string\[]/);
  assert.match(types, /loading: AdminModuleLoadingPolicy/);
  assert.match(types, /productModule: AdminProductModuleManifest/);
  assert.match(routeManifest, /adminProductModules/);
  assert.match(routeManifest, /resolveAdminProductModule/);
  assert.match(routeManifest, /pluginKind: 'admin-module'/);
  assert.match(routeManifest, /requiredPermissions:/);
  assert.match(routeManifest, /capabilityTags:/);
  assert.match(routeManifest, /strategy: 'lazy'/);
  assert.match(routeManifest, /prefetch: 'intent'/);
  assert.match(coreIndex, /adminProductModules/);
  assert.match(coreIndex, /resolveAdminProductModule/);
});

test('core store exposes theme and sidebar shell state', () => {
  const core = read('packages/sdkwork-router-admin-core/src/index.tsx');
  const types = read('packages/sdkwork-router-admin-types/src/index.ts');
  const store = read('packages/sdkwork-router-admin-core/src/store.ts');
  const routePaths = read('packages/sdkwork-router-admin-core/src/routePaths.ts');

  assert.match(types, /ThemeMode/);
  assert.match(types, /ThemeColor/);
  assert.match(types, /AdminThemePreference/);
  assert.match(types, /AdminSidebarItemKey/);
  assert.match(store, /themeMode/);
  assert.match(store, /themeColor/);
  assert.match(store, /sidebarWidth/);
  assert.match(store, /toggleSidebar/);
  assert.match(store, /sidebarCollapsePreference/);
  assert.match(store, /resolveAutoSidebarCollapsed/);
  assert.match(store, /hiddenSidebarItems/);
  assert.match(core, /useAdminAppStore/);
  assert.match(routePaths, /SETTINGS/);
  assert.match(routePaths, /AUTH/);
  assert.match(routePaths, /LOGIN/);
  assert.match(routePaths, /REGISTER/);
  assert.match(routePaths, /FORGOT_PASSWORD/);
});

test('admin workbench delegates snapshot derivation to a dedicated module', () => {
  const workbench = read('packages/sdkwork-router-admin-core/src/workbench.tsx');
  const snapshotBuilder = read(
    'packages/sdkwork-router-admin-core/src/workbenchSnapshot.ts',
  );

  assert.match(workbench, /listApiKeyGroups/);
  assert.match(workbench, /apiKeyGroups/);
  assert.match(workbench, /from '\.\/workbenchSnapshot'/);
  assert.doesNotMatch(workbench, /function buildManagedUsers/);
  assert.doesNotMatch(workbench, /function buildOverviewMetrics/);
  assert.doesNotMatch(workbench, /function buildAlerts/);
  assert.doesNotMatch(workbench, /function buildSnapshot/);
  assert.match(snapshotBuilder, /export const emptySnapshot/);
  assert.match(snapshotBuilder, /export function buildManagedUsers/);
  assert.match(snapshotBuilder, /export function buildOverviewMetrics/);
  assert.match(snapshotBuilder, /export function buildAlerts/);
  assert.match(snapshotBuilder, /export function buildSnapshot/);
});

test('admin workbench delegates mutable control-plane actions to a dedicated module', () => {
  const workbench = read('packages/sdkwork-router-admin-core/src/workbench.tsx');
  const actions = read('packages/sdkwork-router-admin-core/src/workbenchActions.ts');

  assert.match(workbench, /from '\.\/workbenchActions'/);
  assert.doesNotMatch(workbench, /async function handleSaveOperatorUser/);
  assert.doesNotMatch(workbench, /async function handleCreateApiKey/);
  assert.doesNotMatch(workbench, /async function handleCreateRateLimitPolicy/);
  assert.doesNotMatch(workbench, /async function handleReloadRuntimes/);
  assert.doesNotMatch(workbench, /async function handleSaveChannelModel/);
  assert.match(actions, /export interface WorkbenchActions/);
  assert.match(actions, /export function createWorkbenchActions/);
  assert.match(actions, /saveOperatorUser/);
  assert.match(actions, /createApiKey/);
  assert.match(actions, /createApiKeyGroup/);
  assert.match(actions, /createRoutingProfile/);
  assert.match(actions, /updateApiKeyGroup/);
  assert.match(actions, /updateApiKeyGroupStatus/);
  assert.match(actions, /deleteApiKeyGroup/);
  assert.match(actions, /handleSaveApiKeyGroup/);
  assert.match(actions, /handleCreateRoutingProfile/);
  assert.match(actions, /handleToggleApiKeyGroup/);
  assert.match(actions, /handleDeleteApiKeyGroup/);
  assert.match(actions, /createRateLimitPolicy/);
  assert.match(actions, /handleCreateRateLimitPolicy/);
  assert.match(actions, /reloadExtensionRuntimes/);
  assert.match(actions, /saveChannelModel/);
});

test('shell package owns router, theme manager, header, sidebar, and settings page integration', () => {
  const shell = read('packages/sdkwork-router-admin-shell/src/index.ts');
  const appRoot = read('packages/sdkwork-router-admin-shell/src/application/app/AppRoot.tsx');
  const themeCss = read('src/theme.css');
  const bootstrap = read(
    'packages/sdkwork-router-admin-shell/src/application/bootstrap/bootstrapShellRuntime.ts',
  );
  const providers = read(
    'packages/sdkwork-router-admin-shell/src/application/providers/AppProviders.tsx',
  );
  const themeManager = read(
    'packages/sdkwork-router-admin-shell/src/application/providers/ThemeManager.tsx',
  );
  const layout = read(
    'packages/sdkwork-router-admin-shell/src/application/layouts/MainLayout.tsx',
  );
  const routePaths = read(
    'packages/sdkwork-router-admin-shell/src/application/router/routePaths.ts',
  );
  const sidebar = read('packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx');
  const header = read('packages/sdkwork-router-admin-shell/src/components/AppHeader.tsx');
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');
  const styles = read('packages/sdkwork-router-admin-shell/src/styles/shell-host.css');

  assert.match(shell, /AppRoot/);
  assert.match(shell, /\.\/styles\/shell-host\.css/);
  assert.match(appRoot, /AppProviders/);
  assert.match(appRoot, /AppRoutes/);
  assert.doesNotMatch(appRoot, /<MainLayout \/>/);
  assert.match(bootstrap, /Promise\.resolve|async function bootstrapShellRuntime/);
  assert.match(providers, /BrowserRouter/);
  assert.match(providers, /AdminI18nProvider/);
  assert.match(themeManager, /createSdkworkTheme/);
  assert.match(themeManager, /data-theme/);
  assert.match(themeManager, /data-sdk-color-mode/);
  assert.doesNotMatch(themeManager, /data-sidebar-collapsed/);
  assert.match(layout, /relative flex h-screen flex-col overflow-hidden/);
  assert.match(layout, /\[background:var\(--admin-shell-background\)\]/);
  assert.match(layout, /Sidebar/);
  assert.match(layout, /AppHeader/);
  assert.match(layout, /admin-shell-content/);
  assert.match(layout, /bg-\[var\(--admin-content-background\)\]/);
  assert.match(layout, /data-sdk-shell="router-admin-desktop"/);
  assert.doesNotMatch(layout, /DesktopShellFrame/);
  assert.doesNotMatch(layout, /AdminShellBrandMark/);
  assert.doesNotMatch(layout, /SDKWork Router Admin/);
  assert.doesNotMatch(layout, /Control plane/);
  assert.doesNotMatch(layout, /AppRoutes/);
  assert.doesNotMatch(layout, /isAdminAuthPath/);
  assert.match(routePaths, /ROUTE_PATHS|ADMIN_ROUTE_PATHS/);
  assert.match(sidebar, /motion\/react/);
  assert.match(sidebar, /sidebar-edge-control/);
  assert.match(sidebar, /PanelLeftOpen/);
  assert.match(sidebar, /ChevronUp/);
  assert.match(sidebar, /\[background:var\(--admin-sidebar-background\)\]/);
  assert.match(sidebar, /bg-primary-500/);
  assert.match(sidebar, /text-primary-400/);
  assert.match(sidebar, /currentSidebarWidth = isSidebarCollapsed \? COLLAPSED_SIDEBAR_WIDTH : resolvedSidebarWidth/);
  assert.match(sidebar, /toggleSidebar/);
  assert.match(sidebar, /settings/);
  assert.doesNotMatch(sidebar, /NavigationRail/);
  assert.doesNotMatch(sidebar, /DropdownMenu/);
  assert.doesNotMatch(sidebar, /AdminShellBrandMark/);
  assert.match(header, /\[background:var\(--admin-header-background\)\]/);
  assert.match(header, /ShellStatus/);
  assert.match(header, /HeaderActionButton/);
  assert.match(header, /data-slot="app-header-leading"/);
  assert.match(header, /data-slot="app-header-brand"/);
  assert.match(header, /data-slot="app-header-trailing"/);
  assert.match(header, /dataSlot="app-header-search"/);
  assert.match(header, /dataSlot="app-header-refresh"/);
  assert.match(header, /t\('Router Admin'\)/);
  assert.match(header, /ROUTE_PATHS\.OVERVIEW/);
  assert.match(header, /32x32\.png/);
  assert.match(header, /import\.meta\.url/);
  assert.match(header, /Ctrl K/);
  assert.match(header, /Refresh/);
  assert.doesNotMatch(header, /Toolbar/);
  assert.doesNotMatch(header, /ToolbarGroup/);
  assert.doesNotMatch(header, /@sdkwork\/ui-pc-react\/components\/ui/);
  assert.match(routes, /AdminLoginPage/);
  assert.match(routes, /<MainLayout>/);
  assert.match(routes, /ROUTE_PATHS\.AUTH/);
  assert.match(routes, /ROUTE_PATHS\.REGISTER/);
  assert.match(routes, /ROUTE_PATHS\.FORGOT_PASSWORD/);
  assert.match(routes, /SettingsPage/);
  assert.match(themeCss, /@source "\.\/";/);
  assert.match(themeCss, /@source "\.\.\/packages";/);
  assert.match(themeCss, /--admin-shell-background/);
  assert.match(themeCss, /--admin-sidebar-background/);
  assert.match(themeCss, /--admin-header-background/);
  assert.match(themeCss, /--admin-sidebar-text:/);
  assert.match(themeCss, /--admin-sidebar-popover-background:/);
  assert.match(themeCss, /--admin-sidebar-edge-background:/);
  assert.match(styles, /admin-shell-host/);
  assert.match(styles, /admin-shell-route-scroll/);
  assert.match(styles, /admin-shell-sidebar-resize-handle/);
  assert.doesNotMatch(styles, /admin-shell-auth-stage/);
});

test('root shell compatibility files forward to application-owned implementations', () => {
  const rootAppRoot = read('packages/sdkwork-router-admin-shell/src/AppRoot.tsx');
  const rootAppProviders = read('packages/sdkwork-router-admin-shell/src/AppProviders.tsx');
  const rootThemeManager = read('packages/sdkwork-router-admin-shell/src/ThemeManager.tsx');

  assert.match(rootAppRoot, /export \{ AppRoot \} from '\.\/application\/app\/AppRoot';/);
  assert.match(rootAppProviders, /export \{ AppProviders \} from '\.\/application\/providers\/AppProviders';/);
  assert.match(rootThemeManager, /export \{ ThemeManager \} from '\.\/application\/providers\/ThemeManager';/);
  assert.doesNotMatch(rootAppProviders, /BrowserRouter/);
  assert.doesNotMatch(rootThemeManager, /useEffect/);
});

test('users module exposes delete capabilities for operator and portal identities', () => {
  const rootRoutes = read('packages/sdkwork-router-admin-shell/src/AppRoutes.tsx');
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');
  const users = read('packages/sdkwork-router-admin-users/src/index.tsx');
  const usersRegistry = read('packages/sdkwork-router-admin-users/src/page/UsersRegistrySection.tsx');
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');

  assert.match(adminApi, /deleteOperatorUser/);
  assert.match(adminApi, /deletePortalUser/);
  assert.match(rootRoutes, /export \{ AppRoutes \} from '\.\/application\/router\/AppRoutes';/);
  assert.match(routes, /onDeleteOperatorUser=/);
  assert.match(routes, /onDeletePortalUser=/);
  assert.match(users, /ConfirmActionDialog/);
  assert.match(usersRegistry, /Delete/);
});

test('tenants module exposes gateway key issuance workflow', () => {
  const rootRoutes = read('packages/sdkwork-router-admin-shell/src/AppRoutes.tsx');
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');
  const tenants = read('packages/sdkwork-router-admin-tenants/src/index.tsx');
  const tenantsRegistry = read('packages/sdkwork-router-admin-tenants/src/page/TenantsRegistrySection.tsx');
  const tenantsDrawer = read('packages/sdkwork-router-admin-tenants/src/page/TenantsDetailDrawer.tsx');
  const apiKeyDialog = read('packages/sdkwork-router-admin-tenants/src/page/ApiKeyDialog.tsx');
  const plaintextDialog = read('packages/sdkwork-router-admin-tenants/src/page/PlaintextApiKeyDialog.tsx');
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');

  assert.match(adminApi, /createApiKey/);
  assert.match(adminApi, /updateApiKeyStatus/);
  assert.match(adminApi, /deleteApiKey/);
  assert.match(rootRoutes, /export \{ AppRoutes \} from '\.\/application\/router\/AppRoutes';/);
  assert.match(routes, /onCreateApiKey=/);
  assert.match(routes, /onUpdateApiKeyStatus=/);
  assert.match(routes, /onDeleteApiKey=/);
  assert.match(tenants, /Issue gateway key/);
  assert.match(tenants, /Gateway posture/);
  assert.match(tenants, /revealedApiKey/);
  assert.match(tenantsRegistry, /active keys/);
  assert.match(tenantsRegistry, /No tenants available/);
  assert.match(tenantsRegistry, /Delete/);
  assert.match(tenantsDrawer, /Issue key/);
  assert.match(apiKeyDialog, /Issue gateway key/);
  assert.match(plaintextDialog, /Plaintext key ready/);
  assert.match(tenants, /revealedApiKey/);
});

test('gateway sidebar routes mount the migrated claw apirouter surfaces', () => {
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');
  const routeManifest = read('packages/sdkwork-router-admin-core/src/routes.ts');
  const routePaths = read('packages/sdkwork-router-admin-core/src/routePaths.ts');
  const apiRouter = read('packages/sdkwork-router-admin-apirouter/src/index.ts');
  const routeConfigPage = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayRoutesPage.tsx',
  );
  const snapshotAnalytics = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/routes/routingSnapshotAnalytics.ts',
  );
  const snapshotDialog = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutingSnapshotsDialog.tsx',
  );

  assert.match(apiRouter, /GatewayAccessPage/);
  assert.match(apiRouter, /GatewayRateLimitsPage/);
  assert.match(apiRouter, /GatewayRoutesPage/);
  assert.match(apiRouter, /GatewayModelMappingsPage/);
  assert.match(apiRouter, /GatewayUsagePage/);
  assert.match(routeManifest, /Api Key/);
  assert.match(routeManifest, /Rate Limits/);
  assert.match(routeManifest, /Route Config/);
  assert.match(routeManifest, /Model Mapping/);
  assert.match(routeManifest, /Usage Records/);
  assert.match(routeManifest, /API Router/);
  assert.match(routePaths, /API_ROUTER_ROOT/);
  assert.match(routePaths, /API_ROUTER_API_KEYS/);
  assert.match(routePaths, /API_ROUTER_RATE_LIMITS/);
  assert.match(routePaths, /API_ROUTER_ROUTE_CONFIG/);
  assert.match(routePaths, /API_ROUTER_MODEL_MAPPING/);
  assert.match(routePaths, /API_ROUTER_USAGE_RECORDS/);
  assert.match(routePaths, /\/api-router'/);
  assert.match(routePaths, /\/api-router\/api-keys/);
  assert.match(routePaths, /\/api-router\/rate-limits/);
  assert.match(routePaths, /\/api-router\/route-config/);
  assert.match(routePaths, /\/api-router\/model-mapping/);
  assert.match(routePaths, /\/api-router\/usage-records/);
  assert.match(routes, /API_ROUTER_ROOT/);
  assert.match(routes, /API_ROUTER_API_KEYS/);
  assert.match(routes, /API_ROUTER_RATE_LIMITS/);
  assert.match(routes, /GatewayAccessPage/);
  assert.match(routes, /GatewayRateLimitsPage/);
  assert.match(routes, /onCreateRateLimitPolicy=/);
  assert.match(routes, /GatewayRoutesPage/);
  assert.match(routes, /onCreateRoutingProfile=/);
  assert.match(routes, /GatewayModelMappingsPage/);
  assert.match(routes, /GatewayUsagePage/);
  assert.match(routeConfigPage, /Manage routing profiles/);
  assert.match(routeConfigPage, /Snapshot evidence/);
  assert.match(routeConfigPage, /GatewayRoutingSnapshotsDialog/);
  assert.match(routeConfigPage, /buildRoutingSnapshotAnalytics/);
  assert.match(routeConfigPage, /GatewayRoutingProfilesDialog/);
  assert.match(snapshotAnalytics, /export function buildRoutingSnapshotAnalytics/);
  assert.match(snapshotAnalytics, /export function buildProviderRoutingImpact/);
  assert.match(snapshotAnalytics, /compiledRoutingSnapshots/);
  assert.match(snapshotAnalytics, /applied_routing_profile_id/);
  assert.match(snapshotDialog, /Compiled snapshots/);
  assert.match(snapshotDialog, /Applied routing profile/);
});

test('gateway access now exposes first-class API key group governance through the admin workspace', () => {
  const types = read('packages/sdkwork-router-admin-types/src/index.ts');
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');
  const workbench = read('packages/sdkwork-router-admin-core/src/workbench.tsx');
  const snapshotBuilder = read(
    'packages/sdkwork-router-admin-core/src/workbenchSnapshot.ts',
  );
  const page = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx');
  const registry = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessRegistrySection.tsx',
  );
  const detailDrawer = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessDetailDrawer.tsx',
  );
  const groupDialog = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayApiKeyGroupsDialog.tsx',
  );

  assert.match(types, /export interface ApiKeyGroupRecord/);
  assert.match(types, /export interface RoutingProfileRecord/);
  assert.match(types, /export interface CompiledRoutingSnapshotRecord/);
  assert.match(types, /export interface BillingEventRecord/);
  assert.match(types, /export interface BillingEventSummary/);
  assert.match(types, /api_key_group_id\?: string \| null/);
  assert.match(types, /apiKeyGroups:/);
  assert.match(types, /routingProfiles:/);
  assert.match(types, /billingEvents:/);
  assert.match(types, /billingEventSummary:/);
  assert.match(adminApi, /listApiKeyGroups/);
  assert.match(adminApi, /createApiKeyGroup/);
  assert.match(adminApi, /updateApiKeyGroup/);
  assert.match(adminApi, /updateApiKeyGroupStatus/);
  assert.match(adminApi, /deleteApiKeyGroup/);
  assert.match(adminApi, /listRoutingProfiles/);
  assert.match(adminApi, /createRoutingProfile/);
  assert.match(adminApi, /listBillingEvents/);
  assert.match(adminApi, /getBillingEventSummary/);
  assert.match(workbench, /listRoutingProfiles/);
  assert.match(workbench, /listBillingEvents/);
  assert.match(workbench, /getBillingEventSummary/);
  assert.match(snapshotBuilder, /routingProfiles:/);
  assert.match(snapshotBuilder, /billingEvents:/);
  assert.match(snapshotBuilder, /billingEventSummary:/);
  assert.match(page, /Manage groups/);
  assert.match(page, /GatewayApiKeyGroupsDialog/);
  assert.match(registry, /API key group/);
  assert.match(detailDrawer, /Group policy/);
  assert.match(groupDialog, /Create group/);
  assert.match(groupDialog, /Routing profile/);
  assert.match(groupDialog, /routingProfiles/);
  assert.match(groupDialog, /No routing profile override/);
});

test('admin tauri bridge exposes native Api Key setup commands for quick setup parity', () => {
  const tauriMain = read('src-tauri/src/main.rs');

  assert.match(tauriMain, /install_api_router_client_setup/);
  assert.match(tauriMain, /list_api_key_instances/);
});

test('overview and traffic modules expose hotspot analytics', () => {
  const overview = read('packages/sdkwork-router-admin-overview/src/index.tsx');
  const traffic = read('packages/sdkwork-router-admin-traffic/src/index.tsx');
  const usage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx');

  assert.match(overview, /Top portal users/);
  assert.match(overview, /Hottest projects/);
  assert.match(traffic, /User traffic leaderboard/);
  assert.match(traffic, /Project hotspots/);
  assert.match(traffic, /Recent window/);
  assert.match(traffic, /Export CSV/);
  assert.match(traffic, /billingEventSummary\.groups/);
  assert.match(traffic, /billingEventSummary\.capabilities/);
  assert.match(traffic, /billingEventSummary\.accounting_modes/);
  assert.match(traffic, /Group chargeback/);
  assert.match(traffic, /Capability mix/);
  assert.match(traffic, /Accounting mode/);
  assert.match(usage, /Billing events/);
  assert.match(usage, /Group chargeback/);
  assert.match(usage, /Capability mix/);
  assert.match(usage, /Accounting mode/);
  assert.doesNotMatch(traffic, /Portal user scope/);
});

test('traffic routing lens keeps compiled snapshot and fallback evidence searchable and inspectable', () => {
  const traffic = read('packages/sdkwork-router-admin-traffic/src/index.tsx');

  assert.match(traffic, /row\.fallback_reason/);
  assert.match(traffic, /row\.compiled_routing_snapshot_id/);
  assert.match(traffic, /'fallback_reason'/);
  assert.match(traffic, /'compiled_routing_snapshot_id'/);
  assert.match(traffic, /Fallback reason/);
  assert.match(traffic, /Compiled snapshot/);
});

test('operations module exposes runtime reload controls', () => {
  const rootRoutes = read('packages/sdkwork-router-admin-shell/src/AppRoutes.tsx');
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');
  const operations = read('packages/sdkwork-router-admin-operations/src/index.tsx');
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');

  assert.match(adminApi, /reloadExtensionRuntimes/);
  assert.match(rootRoutes, /export \{ AppRoutes \} from '\.\/application\/router\/AppRoutes';/);
  assert.match(routes, /onReloadRuntimes=/);
  assert.match(operations, /Reload all runtimes/);
  assert.match(operations, /Targeted reload/);
  assert.match(operations, /Run targeted reload/);
  assert.match(operations, /Operational posture/);
});

test('catalog module exposes unified directory workflow and provider credential lifecycle management', () => {
  const rootRoutes = read('packages/sdkwork-router-admin-shell/src/AppRoutes.tsx');
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');
  const catalog = read('packages/sdkwork-router-admin-catalog/src/index.tsx');
  const catalogRegistry = read(
    'packages/sdkwork-router-admin-catalog/src/page/CatalogRegistrySection.tsx',
  );
  const catalogDrawer = read(
    'packages/sdkwork-router-admin-catalog/src/page/CatalogDetailDrawer.tsx',
  );
  const catalogChannelModelDialog = read(
    'packages/sdkwork-router-admin-catalog/src/page/CatalogChannelModelDialog.tsx',
  );
  const catalogModelPriceDialog = read(
    'packages/sdkwork-router-admin-catalog/src/page/CatalogModelPriceDialog.tsx',
  );
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');
  const types = read('packages/sdkwork-router-admin-types/src/index.ts');

  assert.match(types, /CredentialRecord/);
  assert.match(types, /credentials:/);
  assert.match(adminApi, /listCredentials/);
  assert.match(adminApi, /saveCredential/);
  assert.match(adminApi, /deleteCredential/);
  assert.match(rootRoutes, /export \{ AppRoutes \} from '\.\/application\/router\/AppRoutes';/);
  assert.match(routes, /onSaveCredential=/);
  assert.match(routes, /onDeleteCredential=/);
  assert.match(catalog, /CatalogDetailDrawer/);
  assert.match(catalog, /Search catalog/);
  assert.match(catalog, /Catalog area/);
  assert.match(catalogRegistry, /Pagination/);
  assert.match(catalogRegistry, /Rotate credential/);
  assert.match(catalogDrawer, /DrawerFooter/);
  assert.match(catalogChannelModelDialog, /Publish model to channel/);
  assert.match(
    catalogModelPriceDialog,
    /Provider-specific pricing rows stay aligned with the selected publication/,
  );
});

test('admin API package exposes API key metadata update support for unified-key parity', () => {
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');

  assert.match(adminApi, /updateApiKey\(/);
  assert.match(adminApi, /\/api-keys\/\$\{encodeURIComponent\(hashedKey\)\}/);
});

test('root app mounts the shell package and keeps shell styling out of the root app', () => {
  const app = read('src/App.tsx');
  const main = read('src/main.tsx');

  assert.match(app, /sdkwork-router-admin-shell/);
  assert.doesNotMatch(main, /framework\.css/);
  assert.match(main, /bootstrapShellRuntime/);
  assert.doesNotMatch(app, /console\//);
});

test('vite config serves static assets from the /admin/ base path', () => {
  const viteConfig = read('vite.config.ts');

  assert.match(viteConfig, /base:\s*'\/admin\/'/);
  assert.match(viteConfig, /manualChunks/);
  assert.match(viteConfig, /react-vendor/);
  assert.match(viteConfig, /radix-vendor/);
  assert.match(viteConfig, /charts-vendor|motion-vendor|icon-vendor/);
});

test('vite browser mode fixes the admin dev server port and proxies the admin API to the managed 9981 backend bind', () => {
  const viteConfig = read('vite.config.ts');

  assert.match(viteConfig, /findReadableModuleResolution/);
  assert.match(viteConfig, /readableExternalFallbackPlugin/);
  assert.match(viteConfig, /loadAdminVitePlugins/);
  assert.match(viteConfig, /defineAdminViteConfig/);
  assert.match(viteConfig, /resolveReadablePackageRoot/);
  assert.match(viteConfig, /react-router-dom/);
  assert.match(viteConfig, /react-router/);
  assert.match(viteConfig, /server:\s*\{/);
  assert.match(viteConfig, /port:\s*5173/);
  assert.match(viteConfig, /strictPort:\s*true/);
  assert.match(viteConfig, /\/api\/admin/);
  assert.match(viteConfig, /127\.0\.0\.1:9981/);
  assert.doesNotMatch(viteConfig, /127\.0\.0\.1:8081/);
});

test('admin root app enables the shared ui tailwind v4 substrate', () => {
  const packageJson = JSON.parse(read('package.json'));
  const viteConfig = read('vite.config.ts');
  const main = read('src/main.tsx');
  const themeCss = read('src/theme.css');
  const sharedUiStyleReferenced =
    viteConfig.includes('@sdkwork/ui-pc-react/styles.css')
    || viteConfig.includes('@sdkwork\\/ui-pc-react\\/styles\\.css');
  const sharedUiThemeReferenced =
    viteConfig.includes('@sdkwork/ui-pc-react/theme')
    || viteConfig.includes('@sdkwork\\/ui-pc-react\\/theme');

  assert.equal(typeof packageJson.devDependencies?.['@tailwindcss/vite'], 'string');
  assert.equal(typeof packageJson.devDependencies?.tailwindcss, 'string');
  assert.equal(typeof packageJson.devDependencies?.['@tailwindcss/typography'], 'string');
  assert.equal(typeof packageJson.dependencies?.['@sdkwork/ui-pc-react'], 'string');
  assert.match(viteConfig, /@tailwindcss\/vite/);
  assert.match(
    viteConfig,
    /plugins:\s*\[readableExternalFallbackPlugin\(\),\s*react\(\),\s*tailwindcss\(\)\]/,
  );
  assert.equal(sharedUiStyleReferenced, true);
  assert.equal(sharedUiThemeReferenced, true);
  assert.match(main, /@sdkwork\/ui-pc-react\/styles\.css/);
  assert.match(main, /\.\/theme\.css/);
  assert.match(themeCss, /@import "tailwindcss";/);
  assert.match(themeCss, /@source "\.\/";/);
  assert.match(themeCss, /@source "\.\.\/packages";/);
});

test('admin routes lazy-load workbench modules behind a loading boundary', () => {
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');

  assert.match(routes, /lazy,\s*Suspense/);
  assert.match(routes, /const OverviewPage = lazy/);
  assert.match(routes, /const UsersPage = lazy/);
  assert.match(routes, /const CatalogPage = lazy/);
  assert.match(routes, /const GatewayAccessPage = lazy/);
  assert.match(routes, /<Suspense fallback=\{<LoadingScreen \/>}/);
  assert.doesNotMatch(routes, /import \{ OverviewPage \} from 'sdkwork-router-admin-overview';/);
  assert.doesNotMatch(routes, /import \{ UsersPage \} from 'sdkwork-router-admin-users';/);
});

test('admin auth package does not ship hardcoded dev credentials or password-prefill copy', () => {
  const auth = read('packages/sdkwork-router-admin-auth/src/index.tsx');

  assert.doesNotMatch(auth, /DEV_ADMIN_CREDENTIALS/);
  assert.doesNotMatch(auth, /admin@sdkwork\.local/);
  assert.doesNotMatch(auth, /ChangeMe123!/);
  assert.doesNotMatch(auth, /Local dev credentials are prefilled/);
});

test('users package does not protect identities by fixed bootstrap email markers', () => {
  const shared = read('packages/sdkwork-router-admin-users/src/page/shared.tsx');
  const detailPanel = read('packages/sdkwork-router-admin-users/src/page/UsersDetailPanel.tsx');

  assert.doesNotMatch(shared, /bootstrapOperatorEmail|bootstrapPortalEmail/);
  assert.doesNotMatch(shared, /admin@sdkwork\.local|portal@sdkwork\.local/);
  assert.match(shared, /user\.id === sessionUserId/);
  assert.doesNotMatch(detailPanel, /Bootstrap operators/);
});

test('legacy commons package is removed and admin core owns shared localization', () => {
  const coreIndex = read('packages/sdkwork-router-admin-core/src/index.tsx');
  const coreI18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');

  assert.equal(
    existsSync(path.join(appRoot, 'packages', 'sdkwork-router-admin-commons', 'package.json')),
    false,
  );
  assert.match(coreIndex, /AdminI18nProvider/);
  assert.match(coreIndex, /useAdminI18n/);
  assert.match(coreI18n, /translateAdminText/);
  assert.match(coreI18n, /ADMIN_LOCALE_OPTIONS/);
});

test('root workspace wires shell and settings packages into dependencies and tsconfig paths', () => {
  const packageJson = JSON.parse(read('package.json'));
  const tsconfig = read('tsconfig.json');

  assert.equal(packageJson.dependencies['sdkwork-router-admin-shell'], 'workspace:*');
  assert.equal(packageJson.dependencies['sdkwork-router-admin-settings'], 'workspace:*');
  assert.equal(packageJson.dependencies['sdkwork-router-admin-commons'], undefined);
  assert.match(tsconfig, /sdkwork-router-admin-shell/);
  assert.match(tsconfig, /sdkwork-router-admin-settings/);
  assert.doesNotMatch(tsconfig, /sdkwork-router-admin-commons/);
});

test('settings center exposes shared settings center theme and sidebar controls', () => {
  const index = read('packages/sdkwork-router-admin-settings/src/index.tsx');
  const settings = read('packages/sdkwork-router-admin-settings/src/Settings.tsx');
  const general = read('packages/sdkwork-router-admin-settings/src/GeneralSettings.tsx');
  const appearance = read('packages/sdkwork-router-admin-settings/src/AppearanceSettings.tsx');
  const navigation = read('packages/sdkwork-router-admin-settings/src/NavigationSettings.tsx');
  const workspace = read('packages/sdkwork-router-admin-settings/src/WorkspaceSettings.tsx');
  const shared = read('packages/sdkwork-router-admin-settings/src/Shared.tsx');

  assert.match(index, /SettingsPage|Settings/);
  assert.match(settings, /SettingsCenter/);
  assert.match(settings, /useSearchParams/);
  assert.match(settings, /Appearance/);
  assert.match(settings, /Navigation/);
  assert.match(settings, /Workspace/);
  assert.match(general, /Language and locale/);
  assert.match(general, /Workspace posture/);
  assert.match(appearance, /theme mode/i);
  assert.match(appearance, /theme color/i);
  assert.match(appearance, /tech-blue/);
  assert.match(appearance, /lobster/);
  assert.match(appearance, /green-tech/);
  assert.match(appearance, /zinc/);
  assert.match(appearance, /violet/);
  assert.match(appearance, /rose/);
  assert.match(navigation, /sidebar/i);
  assert.match(workspace, /content region|right canvas|workspace/i);
  assert.match(shared, /SettingsBadge/);
  assert.match(shared, /SettingsSummaryCard/);
  assert.match(shared, /SettingsChoiceButton/);
});

test('shared ui theme import plus shell host stylesheet define the admin shell contract', () => {
  const main = read('src/main.tsx');
  const themeCss = read('src/theme.css');
  const themeManager = read('packages/sdkwork-router-admin-shell/src/application/providers/ThemeManager.tsx');
  const shellHost = read('packages/sdkwork-router-admin-shell/src/styles/shell-host.css');

  assert.match(main, /@sdkwork\/ui-pc-react\/styles\.css/);
  assert.match(main, /\.\/theme\.css/);
  assert.match(themeCss, /\[data-theme="tech-blue"\]/);
  assert.match(themeCss, /\[data-theme="lobster"\]/);
  assert.match(themeCss, /:root\.dark/);
  assert.match(themeManager, /root\.setAttribute\('data-theme', themeColor\)/);
  assert.match(themeManager, /root\.setAttribute\('data-sdk-color-mode', colorMode\)/);
  assert.match(shellHost, /admin-shell-host/);
  assert.match(shellHost, /admin-shell-content/);
  assert.match(shellHost, /admin-shell-route-scroll/);
  assert.match(shellHost, /admin-shell-sidebar-resize-handle/);
  assert.match(shellHost, /data-sdk-shell='router-admin-desktop'/);
  assert.doesNotMatch(shellHost, /admin-shell-auth-stage/);
});
