import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
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
  const packageJson = JSON.parse(read('package.json'));

  assert.equal(typeof packageJson.scripts?.dev, 'string');
  assert.equal(typeof packageJson.scripts?.build, 'string');
  assert.equal(typeof packageJson.scripts?.typecheck, 'string');
  assert.equal(typeof packageJson.scripts?.preview, 'string');
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
  const routePaths = read('packages/sdkwork-router-portal-core/src/application/router/routePaths.ts');
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');

  assert.match(portalTypes, /'gateway'/);
  assert.match(routes, /gateway/);
  assert.match(routes, /Gateway/);
  assert.match(routes, /dashboard/);
  assert.match(routes, /routing/);
  assert.match(routes, /api-keys/);
  assert.match(routes, /usage/);
  assert.match(routes, /user/);
  assert.match(routes, /credits/);
  assert.match(routes, /billing/);
  assert.match(routes, /account/);
  assert.match(routePaths, /gateway: '\/gateway'/);
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
    path.join('components', 'AppHeader.tsx'),
    path.join('components', 'Sidebar.tsx'),
    path.join('components', 'ConfigCenter.tsx'),
    path.join('store', 'usePortalShellStore.ts'),
    path.join('lib', 'portalPreferences.ts'),
  ]) {
    assert.equal(existsSync(path.join(coreRoot, relativePath)), true, `missing ${relativePath}`);
  }
});

test('root app uses its own theme and does not depend on console/', () => {
  const app = read('src/App.tsx');

  assert.match(app, /import '\.\/theme\.css';/);
  assert.doesNotMatch(app, /console\//);
});

test('vite config serves static assets from the /portal/ base path', () => {
  const viteConfig = read('vite.config.ts');

  assert.match(viteConfig, /base:\s*'\/portal\/'/);
});
