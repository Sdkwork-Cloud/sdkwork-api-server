import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal core exposes a shared auth store with claw-style lifecycle actions', () => {
  const authStorePath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-core',
    'src',
    'store',
    'usePortalAuthStore.ts',
  );

  assert.equal(existsSync(authStorePath), true);

  const authStore = read('packages/sdkwork-router-portal-core/src/store/usePortalAuthStore.ts');

  assert.match(authStore, /isAuthenticated/);
  assert.match(authStore, /isBootstrapping/);
  assert.match(authStore, /signIn/);
  assert.match(authStore, /register/);
  assert.match(authStore, /signOut/);
  assert.match(authStore, /hydrate/);
  assert.match(authStore, /syncWorkspace/);
  assert.match(authStore, /syncDashboard/);
  assert.match(authStore, /onPortalSessionExpired/);
  assert.match(authStore, /persistPortalSessionToken/);
  assert.match(authStore, /clearPortalSessionToken/);
});

test('portal anonymous route contract includes forgot-password parity', () => {
  const types = read('packages/sdkwork-router-portal-types/src/index.ts');
  const routePaths = read('packages/sdkwork-router-portal-core/src/application/router/routePaths.ts');

  assert.match(types, /'forgot-password'/);
  assert.match(routePaths, /'forgot-password':\s*'\/forgot-password'/);
});

test('portal routes use claw-style auth redirects and redirect restore', () => {
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');

  assert.match(appRoutes, /path="auth"/);
  assert.match(appRoutes, /buildAuthHref/);
  assert.match(appRoutes, /URLSearchParams/);
  assert.match(appRoutes, /params\.set\('redirect', redirectTarget\)/);
  assert.match(appRoutes, /requestedTarget/);
  assert.match(appRoutes, /PORTAL_ROUTE_PATHS\['forgot-password'\]/);
});

test('portal auth package exposes a single router-driven auth page', () => {
  const authEntry = read('packages/sdkwork-router-portal-auth/src/index.tsx');
  const authPagePath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-auth',
    'src',
    'pages',
    'AuthPage.tsx',
  );

  assert.equal(existsSync(authPagePath), true);
  assert.match(authEntry, /AuthPage/);

  const authPage = read('packages/sdkwork-router-portal-auth/src/pages/AuthPage.tsx');
  assert.match(authPage, /useLocation/);
  assert.match(authPage, /useNavigate/);
  assert.match(authPage, /useSearchParams/);
  assert.match(authPage, /location\.pathname/);
  assert.match(authPage, /signIn/);
  assert.match(authPage, /register/);
  assert.match(authPage, /forgot-password/);
  assert.doesNotMatch(authPage, /onNavigate\('login'\)|onNavigate\('register'\)/);
});

test('portal auth visuals follow the claw-style split card layout instead of the custom portal story shell', () => {
  const authPage = read('packages/sdkwork-router-portal-auth/src/pages/AuthPage.tsx');
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');

  assert.match(authPage, /QrCode/);
  assert.match(authPage, /Github/);
  assert.match(authPage, /Chrome/);
  assert.match(authPage, /Smartphone/);
  assert.match(authPage, /Button/);
  assert.match(authPage, /Input/);
  assert.match(authPage, /Label/);
  assert.match(authPage, /max-w-4xl/);
  assert.match(authPage, /rounded-3xl/);
  assert.match(authPage, /bg-zinc-50/);
  assert.match(authPage, /dark:bg-zinc-950/);
  assert.match(authPage, /md:flex-row/);
  assert.match(authPage, /md:w-2\/5/);
  assert.match(authPage, /md:w-3\/5/);
  assert.match(authPage, /withRedirect\('/);
  assert.doesNotMatch(authPage, /AuthShell/);
  assert.doesNotMatch(appRoutes, /function AuthLayout/);
});

test('portal shell profile dock is wired to shared auth state instead of logout-only props', () => {
  const sidebar = read('packages/sdkwork-router-portal-core/src/components/Sidebar.tsx');
  const profileDock = read('packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx');
  const app = read('packages/sdkwork-router-portal-core/src/application/app/PortalProductApp.tsx');

  assert.match(sidebar, /usePortalAuthStore/);
  assert.match(profileDock, /usePortalAuthStore/);
  assert.match(profileDock, /isAuthenticated/);
  assert.match(profileDock, /signOut/);
  assert.match(app, /usePortalAuthStore/);
  assert.doesNotMatch(app, /const \[authenticated, setAuthenticated\]/);
});
