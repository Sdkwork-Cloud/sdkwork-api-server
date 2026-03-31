import { lazy, Suspense, type ReactNode } from 'react';
import { Navigate, Route, Routes, useLocation, useNavigate } from 'react-router-dom';
import { useAdminI18n } from 'sdkwork-router-admin-commons';

import { AdminLoginPage } from 'sdkwork-router-admin-auth';
import {
  ADMIN_ROUTE_PATHS,
  adminRoutePathByKey,
  isAdminAuthPath,
  useAdminWorkbench,
} from 'sdkwork-router-admin-core';
import type { AdminRouteKey } from 'sdkwork-router-admin-types';

import { ROUTE_PATHS } from './routePaths';

const OverviewPage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-overview')).OverviewPage,
}));
const UsersPage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-users')).UsersPage,
}));
const TenantsPage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-tenants')).TenantsPage,
}));
const CouponsPage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-coupons')).CouponsPage,
}));
const GatewayAccessPage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-apirouter')).GatewayAccessPage,
}));
const GatewayRoutesPage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-apirouter')).GatewayRoutesPage,
}));
const GatewayModelMappingsPage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-apirouter')).GatewayModelMappingsPage,
}));
const GatewayUsagePage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-apirouter')).GatewayUsagePage,
}));
const CatalogPage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-catalog')).CatalogPage,
}));
const TrafficPage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-traffic')).TrafficPage,
}));
const OperationsPage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-operations')).OperationsPage,
}));
const SettingsPage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-settings')).SettingsPage,
}));

function PageFrame({
  children,
  routeKey,
}: {
  children: ReactNode;
  routeKey: string;
}) {
  return (
    <div key={routeKey} className="adminx-page-frame">
      <div className="adminx-page-frame-shell">
        <div className="adminx-page-frame-scroll">{children}</div>
      </div>
    </div>
  );
}

function LoadingScreen() {
  const { t } = useAdminI18n();

  return (
    <div className="adminx-shell-loading">
      <div className="adminx-shell-loading-orb" />
      <strong>{t('Synchronizing operator workspace...')}</strong>
      <span>{t('Restoring theme, session, and live control-plane state.')}</span>
    </div>
  );
}

function resolveRedirectTarget(rawTarget: string | null) {
  if (!rawTarget || !rawTarget.startsWith('/')) {
    return ROUTE_PATHS.OVERVIEW;
  }

  if (rawTarget === ROUTE_PATHS.ROOT || isAdminAuthPath(rawTarget)) {
    return ROUTE_PATHS.OVERVIEW;
  }

  if (rawTarget === ROUTE_PATHS.API_ROUTER_ROOT) {
    return ROUTE_PATHS.API_ROUTER_API_KEYS;
  }

  return rawTarget;
}

function withRedirect(pathname: string, rawTarget: string | null) {
  const redirectTarget = resolveRedirectTarget(rawTarget);
  if (redirectTarget === ROUTE_PATHS.OVERVIEW) {
    return pathname;
  }

  const params = new URLSearchParams();
  params.set('redirect', redirectTarget);
  return `${pathname}?${params.toString()}`;
}

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const location = useLocation();
  const { authResolved, sessionUser } = useAdminWorkbench();

  if (!authResolved) {
    return <LoadingScreen />;
  }

  if (!sessionUser) {
    return (
      <Navigate
        to={withRedirect(ROUTE_PATHS.LOGIN, `${location.pathname}${location.search}`)}
        replace
      />
    );
  }

  return <>{children}</>;
}

function ProtectedPage({
  children,
  routeKey,
}: {
  children: ReactNode;
  routeKey: string;
}) {
  return (
    <ProtectedRoute>
      <PageFrame routeKey={routeKey}>
        <Suspense fallback={<LoadingScreen />}>{children}</Suspense>
      </PageFrame>
    </ProtectedRoute>
  );
}

export function AppRoutes() {
  const location = useLocation();
  const navigate = useNavigate();
  const {
    authResolved,
    handleLogin,
    sessionUser,
    snapshot,
    status,
    loading,
    refreshWorkspace,
  } = useAdminWorkbench();
  const {
    handleCreateApiKey,
    handleUpdateApiKey,
    handleDeleteApiKey,
    handleDeleteChannel,
    handleDeleteChannelModel,
    handleDeleteCoupon,
    handleDeleteCredential,
    handleDeleteModel,
    handleDeleteModelPrice,
    handleDeleteOperatorUser,
    handleDeletePortalUser,
    handleDeleteProject,
    handleDeleteProvider,
    handleDeleteTenant,
    handleReloadRuntimes,
    handleSaveChannel,
    handleSaveChannelModel,
    handleSaveCoupon,
    handleSaveCredential,
    handleSaveModel,
    handleSaveModelPrice,
    handleSaveOperatorUser,
    handleSavePortalUser,
    handleSaveProject,
    handleSaveProvider,
    handleSaveTenant,
    handleToggleCoupon,
    handleToggleOperatorUser,
    handleTogglePortalUser,
    handleUpdateApiKeyStatus,
  } = useAdminWorkbench();

  const navigateToRoute = (routeKey: AdminRouteKey) => {
    navigate(adminRoutePathByKey[routeKey]);
  };

  const authRouteElement = !authResolved ? (
    <LoadingScreen />
  ) : (
    <AdminLoginPage
      status={status}
      loading={loading}
      isAuthenticated={Boolean(sessionUser)}
      onLogin={handleLogin}
    />
  );

  return (
    <Routes>
      <Route
        path={ROUTE_PATHS.AUTH}
        element={
          <Navigate
            to={withRedirect(ROUTE_PATHS.LOGIN, new URLSearchParams(location.search).get('redirect'))}
            replace
          />
        }
      />
      <Route
        path={ROUTE_PATHS.LOGIN}
        element={authRouteElement}
      />
      <Route
        path={ROUTE_PATHS.REGISTER}
        element={authRouteElement}
      />
      <Route
        path={ROUTE_PATHS.FORGOT_PASSWORD}
        element={authRouteElement}
      />
      <Route
        path={ROUTE_PATHS.OVERVIEW}
        element={
          <ProtectedPage routeKey={location.pathname}>
            <OverviewPage snapshot={snapshot} onNavigate={navigateToRoute} />
          </ProtectedPage>
        }
      />
      <Route
        path={ROUTE_PATHS.USERS}
        element={
          <ProtectedPage routeKey={location.pathname}>
            <UsersPage
              snapshot={snapshot}
              onSaveOperatorUser={handleSaveOperatorUser}
              onSavePortalUser={handleSavePortalUser}
              onToggleOperatorUser={handleToggleOperatorUser}
              onTogglePortalUser={handleTogglePortalUser}
              onDeleteOperatorUser={handleDeleteOperatorUser}
              onDeletePortalUser={handleDeletePortalUser}
            />
          </ProtectedPage>
        }
      />
      <Route
        path={ROUTE_PATHS.TENANTS}
        element={
          <ProtectedPage routeKey={location.pathname}>
            <TenantsPage
              snapshot={snapshot}
              onSaveTenant={handleSaveTenant}
              onSaveProject={handleSaveProject}
              onCreateApiKey={handleCreateApiKey}
              onUpdateApiKeyStatus={handleUpdateApiKeyStatus}
              onDeleteApiKey={handleDeleteApiKey}
              onDeleteTenant={handleDeleteTenant}
              onDeleteProject={handleDeleteProject}
            />
          </ProtectedPage>
        }
      />
      <Route
        path={ROUTE_PATHS.COUPONS}
        element={
          <ProtectedPage routeKey={location.pathname}>
            <CouponsPage
              snapshot={snapshot}
              onSaveCoupon={handleSaveCoupon}
              onToggleCoupon={handleToggleCoupon}
              onDeleteCoupon={handleDeleteCoupon}
            />
          </ProtectedPage>
        }
      />
      <Route
        path={ROUTE_PATHS.API_ROUTER_ROOT}
        element={<Navigate to={ROUTE_PATHS.API_ROUTER_API_KEYS} replace />}
      />
      <Route
        path={ROUTE_PATHS.API_ROUTER_API_KEYS}
        element={
          <ProtectedPage routeKey={location.pathname}>
            <GatewayAccessPage
              snapshot={snapshot}
              onRefreshWorkspace={refreshWorkspace}
              onCreateApiKey={handleCreateApiKey}
              onUpdateApiKey={handleUpdateApiKey}
              onUpdateApiKeyStatus={handleUpdateApiKeyStatus}
              onDeleteApiKey={handleDeleteApiKey}
            />
          </ProtectedPage>
        }
      />
      <Route
        path={ROUTE_PATHS.API_ROUTER_ROUTE_CONFIG}
        element={
          <ProtectedPage routeKey={location.pathname}>
            <GatewayRoutesPage
              snapshot={snapshot}
              onRefreshWorkspace={refreshWorkspace}
              onSaveProvider={handleSaveProvider}
              onDeleteProvider={handleDeleteProvider}
            />
          </ProtectedPage>
        }
      />
      <Route
        path={ROUTE_PATHS.API_ROUTER_MODEL_MAPPING}
        element={
          <ProtectedPage routeKey={location.pathname}>
            <GatewayModelMappingsPage snapshot={snapshot} />
          </ProtectedPage>
        }
      />
      <Route
        path={ROUTE_PATHS.API_ROUTER_USAGE_RECORDS}
        element={
          <ProtectedPage routeKey={location.pathname}>
            <GatewayUsagePage
              snapshot={snapshot}
              onRefreshWorkspace={refreshWorkspace}
            />
          </ProtectedPage>
        }
      />
      <Route
        path={ROUTE_PATHS.CATALOG}
        element={
          <ProtectedPage routeKey={location.pathname}>
            <CatalogPage
              snapshot={snapshot}
              onSaveChannel={handleSaveChannel}
              onSaveProvider={handleSaveProvider}
              onSaveCredential={handleSaveCredential}
              onSaveModel={handleSaveModel}
              onSaveChannelModel={handleSaveChannelModel}
              onSaveModelPrice={handleSaveModelPrice}
              onDeleteChannel={handleDeleteChannel}
              onDeleteProvider={handleDeleteProvider}
              onDeleteCredential={handleDeleteCredential}
              onDeleteModel={handleDeleteModel}
              onDeleteChannelModel={handleDeleteChannelModel}
              onDeleteModelPrice={handleDeleteModelPrice}
            />
          </ProtectedPage>
        }
      />
      <Route
        path={ROUTE_PATHS.TRAFFIC}
        element={
          <ProtectedPage routeKey={location.pathname}>
            <TrafficPage snapshot={snapshot} />
          </ProtectedPage>
        }
      />
      <Route
        path={ROUTE_PATHS.OPERATIONS}
        element={
          <ProtectedPage routeKey={location.pathname}>
            <OperationsPage
              snapshot={snapshot}
              onReloadRuntimes={handleReloadRuntimes}
            />
          </ProtectedPage>
        }
      />
      <Route
        path={ROUTE_PATHS.SETTINGS}
        element={
          <ProtectedPage routeKey={location.pathname}>
            <SettingsPage />
          </ProtectedPage>
        }
      />
      <Route path={ADMIN_ROUTE_PATHS.ROOT} element={<Navigate to={ROUTE_PATHS.OVERVIEW} replace />} />
      <Route
        path="*"
        element={<Navigate to={sessionUser ? ROUTE_PATHS.OVERVIEW : ROUTE_PATHS.LOGIN} replace />}
      />
    </Routes>
  );
}
