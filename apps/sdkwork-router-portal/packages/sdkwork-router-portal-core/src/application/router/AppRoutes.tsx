import { lazy, Suspense, type ReactNode } from 'react';
import { Navigate, Route, Routes, useNavigate } from 'react-router-dom';
import type {
  PortalAuthSession,
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
const PortalLoginPage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-auth')).PortalLoginPage,
}));
const PortalRegisterPage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-auth')).PortalRegisterPage,
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

function AuthLayout({ children }: { children: ReactNode }) {
  return (
    <div className="relative flex min-h-screen overflow-hidden [background:var(--portal-shell-background)] font-sans text-[var(--portal-text-primary)] transition-colors duration-300">
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute inset-x-0 top-0 h-40 bg-[radial-gradient(circle_at_top,rgb(var(--portal-accent-rgb)_/_0.16),transparent_68%)]" />
      </div>
      <main className="relative z-10 flex-1 overflow-auto">{children}</main>
    </div>
  );
}

export function AppRoutes({
  authenticated,
  bootStatus,
  bootstrapped,
  dashboardSnapshot,
  onAuthenticated,
  onLogout,
  pulseDetail,
  pulseStatus,
  pulseTitle,
  pulseTone,
  workspace,
}: {
  authenticated: boolean;
  bootStatus: string;
  bootstrapped: boolean;
  dashboardSnapshot: PortalDashboardSummary | null;
  onAuthenticated: (session: PortalAuthSession) => void;
  onLogout: () => void;
  pulseDetail: string;
  pulseStatus: string;
  pulseTitle: string;
  pulseTone: 'accent' | 'positive' | 'warning';
  workspace: PortalWorkspaceSummary | null;
}) {
  const navigate = useNavigate();

  function navigateToRoute(routeKey: Parameters<typeof resolvePortalPath>[0]) {
    navigate(resolvePortalPath(routeKey));
  }

  function renderProtectedRoute(routeKey: PortalRouteKey) {
    switch (routeKey) {
      case 'dashboard':
        return <PortalDashboardPage initialSnapshot={dashboardSnapshot} onNavigate={navigateToRoute} />;
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
          element={(
            <Navigate
              replace
              to={authenticated ? PORTAL_ROUTE_PATHS.dashboard : PORTAL_ROUTE_PATHS.login}
            />
          )}
          path=""
        />
        <Route
          element={
            authenticated ? (
              <Navigate replace to={PORTAL_ROUTE_PATHS.dashboard} />
            ) : (
              <AuthLayout>
                <PortalLoginPage onAuthenticated={onAuthenticated} onNavigate={navigateToRoute} />
              </AuthLayout>
            )
          }
          path={toRouteElementPath(PORTAL_ROUTE_PATHS.login)}
        />
        <Route
          element={
            authenticated ? (
              <Navigate replace to={PORTAL_ROUTE_PATHS.dashboard} />
            ) : (
              <AuthLayout>
                <PortalRegisterPage onAuthenticated={onAuthenticated} onNavigate={navigateToRoute} />
              </AuthLayout>
            )
          }
          path={toRouteElementPath(PORTAL_ROUTE_PATHS.register)}
        />
        {(
          [
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
                <MainLayout
                  activeRoute={routeKey}
                  onLogout={onLogout}
                  pulseDetail={pulseDetail}
                  pulseStatus={pulseStatus}
                  pulseTitle={pulseTitle}
                  pulseTone={pulseTone}
                  workspace={workspace}
                >
                  {renderProtectedRoute(routeKey)}
                </MainLayout>
              ) : (
                <Navigate replace to={PORTAL_ROUTE_PATHS.login} />
              )
            }
            key={routeKey}
            path={toRouteElementPath(resolvePortalPath(routeKey))}
          />
        ))}
        <Route
          element={(
            <Navigate
              replace
              to={authenticated ? PORTAL_ROUTE_PATHS.dashboard : PORTAL_ROUTE_PATHS.login}
            />
          )}
          path="*"
        />
      </Routes>
    </Suspense>
  );
}
