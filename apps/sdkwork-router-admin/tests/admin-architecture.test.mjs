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
  'sdkwork-router-admin-commons',
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
  const packageJson = JSON.parse(read('package.json'));

  assert.equal(typeof packageJson.scripts?.dev, 'string');
  assert.equal(typeof packageJson.scripts?.build, 'string');
  assert.equal(typeof packageJson.scripts?.typecheck, 'string');
  assert.equal(typeof packageJson.scripts?.preview, 'string');
  assert.equal(typeof packageJson.scripts?.['tauri:dev'], 'string');
  assert.equal(typeof packageJson.scripts?.['tauri:build'], 'string');
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

test('shell package owns router, theme manager, header, sidebar, and settings page integration', () => {
  const shell = read('packages/sdkwork-router-admin-shell/src/index.ts');
  const appRoot = read('packages/sdkwork-router-admin-shell/src/application/app/AppRoot.tsx');
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
  const styles = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(shell, /AppRoot/);
  assert.match(shell, /\.\/styles\/index\.css/);
  assert.match(appRoot, /AppProviders/);
  assert.match(bootstrap, /Promise\.resolve|async function bootstrapShellRuntime/);
  assert.match(providers, /BrowserRouter/);
  assert.match(themeManager, /data-theme/);
  assert.doesNotMatch(themeManager, /data-sidebar-collapsed/);
  assert.match(layout, /Sidebar/);
  assert.match(layout, /AppHeader/);
  assert.match(routePaths, /ROUTE_PATHS|ADMIN_ROUTE_PATHS/);
  assert.match(sidebar, /toggleSidebar/);
  assert.match(sidebar, /settings/);
  assert.match(header, /data-slot="app-header-leading"/);
  assert.match(header, /data-slot="app-header-search"/);
  assert.doesNotMatch(header, /ShellStatus/);
  assert.doesNotMatch(header, /data-slot="app-header-center"/);
  assert.doesNotMatch(header, /data-slot="app-header-trailing"/);
  assert.match(routes, /AdminLoginPage/);
  assert.match(routes, /ROUTE_PATHS\.AUTH/);
  assert.match(routes, /ROUTE_PATHS\.REGISTER/);
  assert.match(routes, /ROUTE_PATHS\.FORGOT_PASSWORD/);
  assert.match(routes, /SettingsPage/);
  assert.match(styles, /admin-shell-settings/);
  assert.match(styles, /adminx-shell-header-main/);
  assert.doesNotMatch(styles, /\.adminx-shell-header-center\b/);
});

test('users module exposes delete capabilities for operator and portal identities', () => {
  const rootRoutes = read('packages/sdkwork-router-admin-shell/src/AppRoutes.tsx');
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');
  const users = read('packages/sdkwork-router-admin-users/src/index.tsx');
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');

  assert.match(adminApi, /deleteOperatorUser/);
  assert.match(adminApi, /deletePortalUser/);
  assert.match(rootRoutes, /export \{ AppRoutes \} from '\.\/application\/router\/AppRoutes';/);
  assert.match(routes, /onDeleteOperatorUser=/);
  assert.match(routes, /onDeletePortalUser=/);
  assert.match(users, /Delete/);
});

test('tenants module exposes gateway key issuance workflow', () => {
  const rootRoutes = read('packages/sdkwork-router-admin-shell/src/AppRoutes.tsx');
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');
  const tenants = read('packages/sdkwork-router-admin-tenants/src/index.tsx');
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');

  assert.match(adminApi, /createApiKey/);
  assert.match(adminApi, /updateApiKeyStatus/);
  assert.match(adminApi, /deleteApiKey/);
  assert.match(rootRoutes, /export \{ AppRoutes \} from '\.\/application\/router\/AppRoutes';/);
  assert.match(routes, /onCreateApiKey=/);
  assert.match(routes, /onUpdateApiKeyStatus=/);
  assert.match(routes, /onDeleteApiKey=/);
  assert.match(tenants, /Issue gateway key/);
  assert.match(tenants, /revealedApiKey/);
  assert.match(tenants, /Api keys/);
  assert.match(tenants, /Delete/);
});

test('gateway sidebar routes mount the migrated claw apirouter surfaces', () => {
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');
  const routeManifest = read('packages/sdkwork-router-admin-core/src/routes.ts');
  const routePaths = read('packages/sdkwork-router-admin-core/src/routePaths.ts');
  const apiRouter = read('packages/sdkwork-router-admin-apirouter/src/index.ts');

  assert.match(apiRouter, /GatewayAccessPage/);
  assert.match(apiRouter, /GatewayRoutesPage/);
  assert.match(apiRouter, /GatewayModelMappingsPage/);
  assert.match(apiRouter, /GatewayUsagePage/);
  assert.match(routeManifest, /Api Key/);
  assert.match(routeManifest, /Route Config/);
  assert.match(routeManifest, /Model Mapping/);
  assert.match(routeManifest, /Usage Records/);
  assert.match(routeManifest, /API Router/);
  assert.match(routePaths, /API_ROUTER_ROOT/);
  assert.match(routePaths, /API_ROUTER_API_KEYS/);
  assert.match(routePaths, /API_ROUTER_ROUTE_CONFIG/);
  assert.match(routePaths, /API_ROUTER_MODEL_MAPPING/);
  assert.match(routePaths, /API_ROUTER_USAGE_RECORDS/);
  assert.match(routePaths, /\/api-router'/);
  assert.match(routePaths, /\/api-router\/api-keys/);
  assert.match(routePaths, /\/api-router\/route-config/);
  assert.match(routePaths, /\/api-router\/model-mapping/);
  assert.match(routePaths, /\/api-router\/usage-records/);
  assert.match(routes, /API_ROUTER_ROOT/);
  assert.match(routes, /API_ROUTER_API_KEYS/);
  assert.match(routes, /GatewayAccessPage/);
  assert.match(routes, /GatewayRoutesPage/);
  assert.match(routes, /GatewayModelMappingsPage/);
  assert.match(routes, /GatewayUsagePage/);
});

test('admin tauri bridge exposes native Api Key setup commands for quick setup parity', () => {
  const tauriMain = read('src-tauri/src/main.rs');

  assert.match(tauriMain, /install_api_router_client_setup/);
  assert.match(tauriMain, /list_api_key_instances/);
});

test('overview and traffic modules expose hotspot analytics', () => {
  const overview = read('packages/sdkwork-router-admin-overview/src/index.tsx');
  const traffic = read('packages/sdkwork-router-admin-traffic/src/index.tsx');

  assert.match(overview, /Top portal users/);
  assert.match(overview, /Hottest projects/);
  assert.match(traffic, /User traffic leaderboard/);
  assert.match(traffic, /Project hotspots/);
  assert.match(traffic, /Recent window/);
  assert.match(traffic, /Export CSV/);
  assert.doesNotMatch(traffic, /Portal user scope/);
});

test('operations module exposes runtime reload controls', () => {
  const rootRoutes = read('packages/sdkwork-router-admin-shell/src/AppRoutes.tsx');
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');
  const operations = read('packages/sdkwork-router-admin-operations/src/index.tsx');
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');

  assert.match(adminApi, /reloadExtensionRuntimes/);
  assert.match(rootRoutes, /export \{ AppRoutes \} from '\.\/application\/router\/AppRoutes';/);
  assert.match(routes, /onReloadRuntimes=/);
  assert.match(operations, /Reload runtimes/);
  assert.match(operations, /Targeted reload/);
});

test('catalog module exposes unified directory workbench and provider credential lifecycle management', () => {
  const rootRoutes = read('packages/sdkwork-router-admin-shell/src/AppRoutes.tsx');
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');
  const catalog = read('packages/sdkwork-router-admin-catalog/src/index.tsx');
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
  assert.match(catalog, /Catalog lane/);
  assert.match(catalog, /Channel focus/);
  assert.match(catalog, /Catalog workbench/);
  assert.match(catalog, /Manage channel models/);
  assert.match(catalog, /Rotate secret/);
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
});

test('admin root app enables the claw-studio tailwind v4 substrate', () => {
  const packageJson = JSON.parse(read('package.json'));
  const viteConfig = read('vite.config.ts');
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.equal(typeof packageJson.devDependencies?.['@tailwindcss/vite'], 'string');
  assert.equal(typeof packageJson.devDependencies?.tailwindcss, 'string');
  assert.equal(typeof packageJson.devDependencies?.['@tailwindcss/typography'], 'string');
  assert.match(viteConfig, /@tailwindcss\/vite/);
  assert.match(viteConfig, /plugins:\s*\[react\(\),\s*tailwindcss\(\)\]/);
  assert.match(theme, /@import "tailwindcss";/);
  assert.match(theme, /@plugin "@tailwindcss\/typography";/);
  assert.match(theme, /@source "\.\.\/\.\.\/\.\.\/\.\.\//);
  assert.match(theme, /@theme\s*\{/);
});

test('admin commons package exposes radix and shadcn-style dependencies', () => {
  const packageJson = JSON.parse(read('packages/sdkwork-router-admin-commons/package.json'));

  assert.equal(typeof packageJson.dependencies?.['@radix-ui/react-dialog'], 'string');
  assert.equal(typeof packageJson.dependencies?.['@radix-ui/react-label'], 'string');
  assert.equal(typeof packageJson.dependencies?.['@radix-ui/react-slot'], 'string');
  assert.equal(typeof packageJson.dependencies?.['class-variance-authority'], 'string');
  assert.equal(typeof packageJson.dependencies?.clsx, 'string');
  assert.equal(typeof packageJson.dependencies?.['tailwind-merge'], 'string');
});

test('root workspace wires shell and settings packages into dependencies and tsconfig paths', () => {
  const packageJson = JSON.parse(read('package.json'));
  const tsconfig = read('tsconfig.json');

  assert.equal(packageJson.dependencies['sdkwork-router-admin-shell'], 'workspace:*');
  assert.equal(packageJson.dependencies['sdkwork-router-admin-settings'], 'workspace:*');
  assert.match(tsconfig, /sdkwork-router-admin-shell/);
  assert.match(tsconfig, /sdkwork-router-admin-settings/);
});

test('settings center exposes claw-studio style theme and sidebar controls', () => {
  const index = read('packages/sdkwork-router-admin-settings/src/index.tsx');
  const settings = read('packages/sdkwork-router-admin-settings/src/Settings.tsx');
  const general = read('packages/sdkwork-router-admin-settings/src/GeneralSettings.tsx');
  const appearance = read('packages/sdkwork-router-admin-settings/src/AppearanceSettings.tsx');
  const navigation = read('packages/sdkwork-router-admin-settings/src/NavigationSettings.tsx');
  const workspace = read('packages/sdkwork-router-admin-settings/src/WorkspaceSettings.tsx');
  const shared = read('packages/sdkwork-router-admin-settings/src/Shared.tsx');

  assert.match(index, /SettingsPage|Settings/);
  assert.match(settings, /useSearchParams/);
  assert.match(settings, /Appearance/);
  assert.match(settings, /Navigation/);
  assert.match(settings, /Workspace/);
  assert.match(general, /shell|workspace|operator/i);
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
  assert.match(shared, /SettingsSection|SettingsShellCard|SettingsNavButton/);
});

test('shell stylesheet defines claw-studio theme tokens and shell selectors', () => {
  const theme = read('packages/sdkwork-router-admin-shell/src/styles/index.css');

  assert.match(theme, /data-theme="tech-blue"/);
  assert.match(theme, /data-theme="lobster"/);
  assert.match(theme, /data-theme="green-tech"/);
  assert.match(theme, /data-theme="zinc"/);
  assert.match(theme, /data-theme="violet"/);
  assert.match(theme, /data-theme="rose"/);
  assert.match(theme, /adminx-shell-sidebar-resize-handle/);
  assert.match(theme, /adminx-shell-sidebar-toggle/);
  assert.match(theme, /admin-shell-settings/);
});
