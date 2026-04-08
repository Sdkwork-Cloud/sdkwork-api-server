import assert from 'node:assert/strict';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function walkFiles(dir, output = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === 'dist' || entry.name === 'node_modules' || entry.name === 'tests') {
        continue;
      }

      walkFiles(fullPath, output);
      continue;
    }

    if (/\.(ts|tsx)$/.test(entry.name)) {
      output.push(fullPath);
    }
  }

  return output;
}

const requiredPackages = [
  'sdkwork-router-portal-types',
  'sdkwork-router-portal-commons',
  'sdkwork-router-portal-core',
  'sdkwork-router-portal-portal-api',
  'sdkwork-router-portal-gateway',
  'sdkwork-router-portal-auth',
  'sdkwork-router-portal-dashboard',
  'sdkwork-router-portal-routing',
  'sdkwork-router-portal-api-keys',
  'sdkwork-router-portal-usage',
  'sdkwork-router-portal-user',
  'sdkwork-router-portal-credits',
  'sdkwork-router-portal-billing',
  'sdkwork-router-portal-account',
];

const requiredBusinessPackages = [
  'sdkwork-router-portal-gateway',
  'sdkwork-router-portal-auth',
  'sdkwork-router-portal-dashboard',
  'sdkwork-router-portal-routing',
  'sdkwork-router-portal-api-keys',
  'sdkwork-router-portal-usage',
  'sdkwork-router-portal-user',
  'sdkwork-router-portal-credits',
  'sdkwork-router-portal-billing',
  'sdkwork-router-portal-account',
];

test('standalone sdkwork-router-portal app root exists', () => {
  assert.equal(existsSync(path.join(appRoot, 'package.json')), true);
  assert.equal(existsSync(path.join(appRoot, 'src', 'App.tsx')), true);
  assert.equal(existsSync(path.join(appRoot, 'src', 'theme.css')), true);
});

test('app root exposes dev, build, typecheck, and preview scripts', () => {
  const packageJsonSource = read('package.json');
  const packageJson = JSON.parse(packageJsonSource);

  assert.equal(typeof packageJson.scripts?.dev, 'string');
  assert.equal(typeof packageJson.scripts?.build, 'string');
  assert.equal(typeof packageJson.scripts?.typecheck, 'string');
  assert.equal(typeof packageJson.scripts?.preview, 'string');
  assert.match(packageJsonSource, /"@types\/node"/);
  assert.match(
    packageJsonSource,
    /"dev": "node \.\.\/\.\.\/scripts\/dev\/run-vite-cli\.mjs --host 0\.0\.0\.0"/,
  );
  assert.match(
    packageJsonSource,
    /"build": "node \.\.\/\.\.\/scripts\/dev\/run-vite-cli\.mjs build"/,
  );
  assert.match(
    packageJsonSource,
    /"typecheck": "tsc --noEmit"/,
  );
  assert.match(
    packageJsonSource,
    /"preview": "node \.\.\/\.\.\/scripts\/dev\/run-vite-cli\.mjs preview --host 0\.0\.0\.0 --port 4174 --strictPort"/,
  );
  assert.match(packageJsonSource, /run-vite-cli\.mjs/);
  assert.doesNotMatch(packageJsonSource, /run-frontend-tool/);
});

test('root app package keeps only bootstrap-level workspace dependencies', () => {
  const packageJson = JSON.parse(read('package.json'));
  const workspaceDependencies = Object.keys(packageJson.dependencies ?? {})
    .filter((dependencyName) => dependencyName.startsWith('sdkwork-router-portal-'))
    .sort();

  assert.deepEqual(workspaceDependencies, ['sdkwork-router-portal-core']);
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

test('dead commerce seed workspace package has been removed', () => {
  assert.equal(
    existsSync(path.join(appRoot, 'packages', 'sdkwork-router-portal-commerce', 'package.json')),
    false,
  );
  assert.doesNotMatch(read('tsconfig.json'), /sdkwork-router-portal-commerce/);
  assert.doesNotMatch(read('README.md'), /sdkwork-router-portal-commerce/);
  assert.doesNotMatch(read('README.md'), /workspace-scoped commerce repository seam/);
  assert.doesNotMatch(read('pnpm-lock.yaml'), /sdkwork-router-portal-commerce/);
});

test('business packages follow the ARCHITECT directory convention', () => {
  for (const packageName of requiredBusinessPackages) {
    const srcRoot = path.join(appRoot, 'packages', packageName, 'src');

    for (const directory of ['types', 'components', 'repository', 'services', 'pages']) {
      assert.equal(
        existsSync(path.join(srcRoot, directory)),
        true,
        `${packageName} is missing src/${directory}`,
      );
    }

    const entryFile = read(path.join('packages', packageName, 'src', 'index.tsx'));
    assert.match(entryFile, /from '\.\/pages'/, `${packageName} entry must re-export from pages`);
  }
});

test('shell route manifest includes the portal product sections', () => {
  const routes = read('packages/sdkwork-router-portal-core/src/routes.ts');
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');
  const routePaths = read('packages/sdkwork-router-portal-core/src/application/router/routePaths.ts');
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');

  assert.match(portalTypes, /PortalRouteGroupKey/);
  assert.match(portalTypes, /'gateway'/);
  assert.match(routes, /gateway/);
  assert.match(routes, /Gateway/);
  assert.match(routes, /dashboard/);
  assert.match(routes, /routing/);
  assert.match(routes, /api-keys/);
  assert.match(routes, /usage/);
  assert.match(routes, /user/);
  assert.match(routes, /key:\s*'credits'/);
  assert.match(routes, /labelKey:\s*'Redeem'/);
  assert.match(routes, /billing/);
  assert.match(routes, /account/);
  assert.match(routes, /group:\s*'operations'/);
  assert.match(routes, /group:\s*'access'/);
  assert.match(routes, /group:\s*'revenue'/);
  assert.match(routePaths, /gateway: '\/console\/gateway'/);
  assert.match(routePaths, /credits: '\/console\/redeem'/);
  assert.match(appRoutes, /case 'credits':/);
  assert.match(appRoutes, /'credits',/);
});

test('portal core exposes a package-first route manifest and shell prefetch derives from it', () => {
  const routeManifest = read(
    'packages/sdkwork-router-portal-core/src/application/router/routeManifest.ts',
  );
  const routePrefetch = read(
    'packages/sdkwork-router-portal-core/src/application/router/routePrefetch.ts',
  );
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');

  assert.match(portalTypes, /PortalRouteModuleId/);
  assert.match(portalTypes, /PortalRouteManifestEntry/);
  assert.match(routeManifest, /portalRouteManifest/);
  assert.match(routeManifest, /moduleId:/);
  assert.match(routeManifest, /sdkwork-router-portal-dashboard/);
  assert.match(routeManifest, /resolvePortalPath/);
  assert.match(routePrefetch, /portalRouteManifest/);
  assert.match(routePrefetch, /loadPortalRouteModule/);
  assert.doesNotMatch(routePrefetch, /\[PORTAL_ROUTE_PATHS\.gateway,\s*\(\)\s*=>\s*import\('sdkwork-router-portal-gateway'\)\]/);
});

test('portal core formalizes product module manifests with capability, permission, and lazy-load metadata', () => {
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const routeManifest = read(
    'packages/sdkwork-router-portal-core/src/application/router/routeManifest.ts',
  );
  const coreIndex = read('packages/sdkwork-router-portal-core/src/index.tsx');

  assert.match(portalTypes, /PortalProductModuleManifest/);
  assert.match(portalTypes, /pluginKind: 'portal-module'/);
  assert.match(portalTypes, /capabilityTags: string\[]/);
  assert.match(portalTypes, /requiredPermissions: string\[]/);
  assert.match(portalTypes, /loading: PortalModuleLoadingPolicy/);
  assert.match(portalTypes, /productModule: PortalProductModuleManifest/);
  assert.match(routeManifest, /portalProductModules/);
  assert.match(routeManifest, /resolvePortalProductModule/);
  assert.match(routeManifest, /pluginKind: 'portal-module'/);
  assert.match(routeManifest, /requiredPermissions:/);
  assert.match(routeManifest, /capabilityTags:/);
  assert.match(routeManifest, /strategy: 'lazy'/);
  assert.match(routeManifest, /prefetch: 'intent'/);
  assert.match(coreIndex, /portalProductModules/);
  assert.match(coreIndex, /resolvePortalProductModule/);
});

test('portal api key contracts are group-aware across types, api client, repository, and page flows', () => {
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');
  const repository = read('packages/sdkwork-router-portal-api-keys/src/repository/index.ts');
  const page = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');

  assert.match(portalTypes, /export interface ApiKeyGroupRecord/);
  assert.match(portalTypes, /api_key_group_id\?: string \| null;/);
  assert.match(portalTypes, /default_routing_profile_id\?: string \| null;/);

  assert.match(portalApi, /listPortalApiKeyGroups/);
  assert.match(portalApi, /createPortalApiKeyGroup/);
  assert.match(portalApi, /updatePortalApiKeyGroup/);
  assert.match(portalApi, /deletePortalApiKeyGroup/);
  assert.match(portalApi, /updatePortalApiKeyGroupStatus/);
  assert.match(portalApi, /listPortalRoutingProfiles/);
  assert.match(portalApi, /api_key_group_id\?: string \| null;/);

  assert.match(repository, /loadPortalApiKeyGroups/);
  assert.match(repository, /loadPortalApiKeyWorkbenchData/);
  assert.match(repository, /loadPortalRoutingProfiles/);
  assert.match(page, /loadPortalApiKeyWorkbenchData/);
  assert.match(page, /loadPortalRoutingProfiles/);
  assert.match(page, /groupId/);
  assert.match(page, /apiKeyGroupId/);
});

test('portal api key page exposes a dedicated group workbench backed by routing profile discovery', () => {
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');
  const page = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');
  const components = read('packages/sdkwork-router-portal-api-keys/src/components/index.tsx');
  const groupsDialog = read(
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyGroupsDialog.tsx',
  );

  assert.match(portalTypes, /export interface RoutingProfileRecord/);
  assert.match(portalApi, /\/routing\/profiles/);
  assert.match(portalApi, /Promise<RoutingProfileRecord\[]>/);
  assert.match(page, /Manage groups/);
  assert.match(page, /PortalApiKeyGroupsDialog/);
  assert.match(page, /issuePortalApiKeyGroup/);
  assert.match(page, /editPortalApiKeyGroup/);
  assert.match(page, /setPortalApiKeyGroupActive/);
  assert.match(page, /removePortalApiKeyGroup/);
  assert.match(components, /PortalApiKeyGroupsDialog/);
  assert.match(groupsDialog, /Create group/);
  assert.match(groupsDialog, /Delete group/);
  assert.match(groupsDialog, /Routing profile/);
});

test('portal routing page exposes a reusable profile workbench backed by workspace-scoped create and list contracts', () => {
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');
  const repository = read('packages/sdkwork-router-portal-routing/src/repository/index.ts');
  const page = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const components = read('packages/sdkwork-router-portal-routing/src/components/index.tsx');
  const dialog = read(
    'packages/sdkwork-router-portal-routing/src/components/PortalRoutingProfilesDialog.tsx',
  );

  assert.match(portalApi, /createPortalRoutingProfile/);
  assert.match(portalApi, /\/routing\/profiles/);
  assert.match(repository, /loadPortalRoutingProfiles/);
  assert.match(repository, /issuePortalRoutingProfile/);
  assert.match(page, /Manage routing profiles/);
  assert.match(page, /PortalRoutingProfilesDialog/);
  assert.match(page, /loadPortalRoutingProfiles/);
  assert.match(page, /issuePortalRoutingProfile/);
  assert.match(components, /PortalRoutingProfilesDialog/);
  assert.match(dialog, /Save current posture/);
  assert.match(dialog, /Use as posture/);
});

test('portal routing page exposes compiled snapshot evidence backed by workspace-scoped snapshot contracts', () => {
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');
  const repository = read('packages/sdkwork-router-portal-routing/src/repository/index.ts');
  const page = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const components = read('packages/sdkwork-router-portal-routing/src/components/index.tsx');
  const dialog = read(
    'packages/sdkwork-router-portal-routing/src/components/PortalRoutingSnapshotsDialog.tsx',
  );

  assert.match(portalTypes, /export interface PortalCompiledRoutingSnapshotRecord/);
  assert.match(portalTypes, /compiled_routing_snapshot_id\?: string \| null;/);
  assert.match(portalTypes, /fallback_reason\?: string \| null;/);
  assert.match(portalApi, /listPortalRoutingSnapshots/);
  assert.match(portalApi, /\/routing\/snapshots/);
  assert.match(repository, /loadPortalRoutingSnapshots/);
  assert.match(page, /View compiled snapshots/);
  assert.match(page, /Open snapshot evidence/);
  assert.match(page, /Fallback posture/);
  assert.match(page, /Compiled snapshot/);
  assert.match(page, /No fallback used/);
  assert.match(page, /No snapshot captured/);
  assert.match(page, /compiled_routing_snapshot_id/);
  assert.match(page, /fallback_reason/);
  assert.match(page, /PortalRoutingSnapshotsDialog/);
  assert.match(page, /loadPortalRoutingSnapshots/);
  assert.match(page, /snapshotSearchQuery/);
  assert.match(components, /PortalRoutingSnapshotsDialog/);
  assert.match(dialog, /Compiled snapshots/);
  assert.match(dialog, /Search compiled snapshots/);
  assert.match(dialog, /suggestedSearchQuery/);
});

test('portal core follows the shell-oriented application structure', () => {
  const coreRoot = path.join(appRoot, 'packages', 'sdkwork-router-portal-core', 'src');

  for (const relativePath of [
    path.join('application', 'app', 'PortalProductApp.tsx'),
    path.join('application', 'layouts', 'MainLayout.tsx'),
    path.join('application', 'providers', 'AppProviders.tsx'),
    path.join('application', 'providers', 'ThemeManager.tsx'),
    path.join('application', 'router', 'AppRoutes.tsx'),
    path.join('application', 'router', 'routeManifest.ts'),
    path.join('application', 'router', 'routePaths.ts'),
    path.join('components', 'PortalDesktopShell.tsx'),
    path.join('components', 'PortalNavigationRail.tsx'),
    path.join('components', 'PortalSettingsCenter.tsx'),
    path.join('store', 'usePortalShellStore.ts'),
    path.join('lib', 'portalPreferences.ts'),
  ]) {
    assert.equal(existsSync(path.join(coreRoot, relativePath)), true, `missing ${relativePath}`);
  }
});

test('root app entry mounts shared theme css and does not depend on console/', () => {
  const app = read('src/App.tsx');
  const main = read('src/main.tsx');

  assert.match(main, /import '@sdkwork\/ui-pc-react\/styles\.css';/);
  assert.match(main, /import '\.\/theme\.css';/);
  assert.doesNotMatch(app, /console\//);
});

test('vite config serves static assets from the /portal/ base path', () => {
  const viteConfig = read('vite.config.ts');

  assert.match(viteConfig, /base:\s*'\/portal\/'/);
});

test('vite browser mode fixes the portal dev server port and proxies both portal and admin APIs to the managed 998x backend binds', () => {
  const viteConfig = read('vite.config.ts');

  assert.match(viteConfig, /server:\s*\{/);
  assert.match(viteConfig, /port:\s*5174/);
  assert.match(viteConfig, /strictPort:\s*true/);
  assert.match(viteConfig, /\/api\/portal/);
  assert.match(viteConfig, /127\.0\.0\.1:9982/);
  assert.match(viteConfig, /\/api\/admin/);
  assert.match(viteConfig, /127\.0\.0\.1:9981/);
  assert.doesNotMatch(viteConfig, /127\.0\.0\.1:8081/);
  assert.doesNotMatch(viteConfig, /127\.0\.0\.1:8082/);
});

test('vite config resolves aliases without CommonJS-only __dirname', () => {
  const viteConfig = read('vite.config.ts');

  assert.doesNotMatch(viteConfig, /__dirname/);
  assert.match(viteConfig, /fileURLToPath\(import\.meta\.url\)|new URL\(/);
});

test('workspace packages declare lucide-react where they directly import icons', () => {
  const frameworkPackageJson = JSON.parse(
    readFileSync(
      path.join(appRoot, '../../../sdkwork-ui/sdkwork-ui-pc-react/package.json'),
      'utf8',
    ),
  );
  const expectedLucideVersion = frameworkPackageJson.dependencies?.['lucide-react'];

  assert.equal(typeof expectedLucideVersion, 'string');

  for (const packageName of requiredPackages) {
    const srcRoot = path.join(appRoot, 'packages', packageName, 'src');
    const sourceFiles = walkFiles(srcRoot);
    const importsLucide = sourceFiles.some((filePath) =>
      /from ['"]lucide-react['"]/.test(readFileSync(filePath, 'utf8')),
    );

    if (!importsLucide) {
      continue;
    }

    const packageJson = JSON.parse(read(path.join('packages', packageName, 'package.json')));

    assert.equal(
      packageJson.dependencies?.['lucide-react'],
      expectedLucideVersion,
      `${packageName} must declare lucide-react directly when it imports icons`,
    );
  }
});
