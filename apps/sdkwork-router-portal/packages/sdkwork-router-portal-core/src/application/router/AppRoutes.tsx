import { lazy, Suspense } from 'react';
import {
  Navigate,
  Route,
  Routes,
  useLocation,
  useNavigate,
  useSearchParams,
} from 'react-router-dom';
import type {
  PortalDashboardSummary,
  PortalRouteKey,
  PortalWorkspaceSummary,
} from 'sdkwork-router-portal-types';

import { MainLayout } from '../layouts/MainLayout';
import { resolvePortalPath } from './routeManifest';
import { PORTAL_ROUTE_PATHS, toRouteElementPath } from './routePaths';

const PortalAccountPage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-account')).PortalAccountPage,
}));
const PortalApiKeysPage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-api-keys')).PortalApiKeysPage,
}));
const PortalAuthPage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-auth')).AuthPage,
}));
const PortalBillingPage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-billing')).PortalBillingPage,
}));
const PortalCreditsPage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-credits')).PortalCreditsPage,
}));
const PortalDashboardPage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-dashboard')).PortalDashboardPage,
}));
const PortalGatewayPage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-gateway')).PortalGatewayPage,
}));
const PortalRoutingPage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-routing')).PortalRoutingPage,
}));
const PortalUserPage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-user')).PortalUserPage,
}));
const PortalUsagePage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-usage')).PortalUsagePage,
}));

function PortalBootScreen({ status }: { status: string }) {
  return (
    <section className="grid min-h-screen place-items-center px-6 py-10">
      <div className="grid w-[min(560px,100%)] gap-4 rounded-[32px] border border-[color:var(--portal-contrast-border)] [background:var(--portal-surface-contrast)] p-8 shadow-[var(--portal-shadow-strong)]">
        <p className="text-xs font-semibold uppercase tracking-[0.24em] text-[var(--portal-text-muted-on-contrast)]">
          Portal Bootstrap
        </p>
        <h1 className="text-4xl font-semibold tracking-tight text-[var(--portal-text-on-contrast)]">
          Restoring workspace access
        </h1>
        <p className="text-sm leading-6 text-[var(--portal-text-muted-on-contrast)]">{status}</p>
      </div>
    </section>
  );
}

function resolveRedirectTarget(rawTarget: string | null): string {
  if (!rawTarget || !rawTarget.startsWith('/')) {
    return PORTAL_ROUTE_PATHS.dashboard;
  }

  if (
    rawTarget === '/auth' ||
    rawTarget === PORTAL_ROUTE_PATHS.login ||
    rawTarget === PORTAL_ROUTE_PATHS.register ||
    rawTarget === PORTAL_ROUTE_PATHS['forgot-password']
  ) {
    return PORTAL_ROUTE_PATHS.dashboard;
  }

  return rawTarget;
}

function buildAuthHref(pathname: string, redirectTarget?: string): string {
  const params = new URLSearchParams();

  if (redirectTarget && redirectTarget !== PORTAL_ROUTE_PATHS.dashboard) {
    params.set('redirect', redirectTarget);
  }

  const query = params.toString();
  return query ? `${pathname}?${query}` : pathname;
}

export function AppRoutes({
  authenticated,
  bootStatus,
  bootstrapped,
  dashboardSnapshot,
  register,
  signIn,
  workspace,
}: {
  authenticated: boolean;
  bootStatus: string;
  bootstrapped: boolean;
  dashboardSnapshot: PortalDashboardSummary | null;
  register: (payload: { name: string; email: string; password: string }) => Promise<unknown>;
  signIn: (credentials: { email: string; password: string }) => Promise<unknown>;
  workspace: PortalWorkspaceSummary | null;
}) {
  const location = useLocation();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const redirectTarget = resolveRedirectTarget(searchParams.get('redirect'));
  const requestedTarget = `${location.pathname}${location.search}`;

  function navigateToRoute(routeKey: Parameters<typeof resolvePortalPath>[0]) {
    navigate(resolvePortalPath(routeKey));
  }

  function renderProtectedRoute(routeKey: PortalRouteKey) {
    switch (routeKey) {
      case 'gateway':
        return <PortalGatewayPage onNavigate={navigateToRoute} />;
      case 'dashboard':
        return (
          <PortalDashboardPage
            initialSnapshot={dashboardSnapshot}
            onNavigate={navigateToRoute}
          />
        );
      case 'routing':
        return <PortalRoutingPage onNavigate={navigateToRoute} />;
      case 'api-keys':
        return <PortalApiKeysPage onNavigate={navigateToRoute} />;
      case 'usage':
        return <PortalUsagePage onNavigate={navigateToRoute} />;
      case 'user':
        return <PortalUserPage onNavigate={navigateToRoute} workspace={workspace} />;
      case 'credits':
        return <PortalCreditsPage onNavigate={navigateToRoute} />;
      case 'billing':
        return <PortalBillingPage onNavigate={navigateToRoute} />;
      case 'account':
        return <PortalAccountPage onNavigate={navigateToRoute} workspace={workspace} />;
      default:
        return null;
    }
  }

  if (!bootstrapped) {
    return <PortalBootScreen status={bootStatus} />;
  }

  return (
    <Suspense fallback={<PortalBootScreen status="Loading portal workspace..." />}>
      <Routes>
        <Route
          element={
            <Navigate
              replace
              to={authenticated ? PORTAL_ROUTE_PATHS.dashboard : PORTAL_ROUTE_PATHS.login}
            />
          }
          path=""
        />
        <Route
          element={
            <Navigate
              replace
              to={buildAuthHref(
                PORTAL_ROUTE_PATHS.login,
                searchParams.get('redirect') ?? undefined,
              )}
            />
          }
          path="auth"
        />
        <Route
          element={
            authenticated ? (
              <Navigate replace to={redirectTarget} />
            ) : (
              <PortalAuthPage register={register} signIn={signIn} />
            )
          }
          path={toRouteElementPath(PORTAL_ROUTE_PATHS.login)}
        />
        <Route
          element={
            authenticated ? (
              <Navigate replace to={redirectTarget} />
            ) : (
              <PortalAuthPage register={register} signIn={signIn} />
            )
          }
          path={toRouteElementPath(PORTAL_ROUTE_PATHS.register)}
        />
        <Route
          element={
            authenticated ? (
              <Navigate replace to={redirectTarget} />
            ) : (
              <PortalAuthPage register={register} signIn={signIn} />
            )
          }
          path={toRouteElementPath(PORTAL_ROUTE_PATHS['forgot-password'])}
        />
        {(
          [
            'gateway',
            'dashboard',
            'routing',
            'api-keys',
            'usage',
            'user',
            'credits',
            'billing',
            'account',
          ] as PortalRouteKey[]
        ).map((routeKey) => (
          <Route
            element={
              authenticated ? (
                <MainLayout workspace={workspace}>
                  {renderProtectedRoute(routeKey)}
                </MainLayout>
              ) : (
                <Navigate
                  replace
                  to={buildAuthHref(PORTAL_ROUTE_PATHS.login, requestedTarget)}
                />
              )
            }
            key={routeKey}
            path={toRouteElementPath(resolvePortalPath(routeKey))}
          />
        ))}
        <Route
          element={
            <Navigate
              replace
              to={authenticated ? PORTAL_ROUTE_PATHS.dashboard : PORTAL_ROUTE_PATHS.login}
            />
          }
          path="*"
        />
      </Routes>
    </Suspense>
  );
}
