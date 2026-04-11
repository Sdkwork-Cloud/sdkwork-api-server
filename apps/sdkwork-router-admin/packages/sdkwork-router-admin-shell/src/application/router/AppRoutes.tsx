import { LoadingBlock } from '@sdkwork/ui-pc-react/components/ui/feedback';
import { lazy, Suspense, type ReactNode } from 'react';
import { Navigate, Route, Routes, useLocation, useNavigate } from 'react-router-dom';

import { AdminLoginPage } from 'sdkwork-router-admin-auth';
import {
  ADMIN_ROUTE_PATHS,
  adminRoutePathByKey,
  isAdminAuthPath,
  useAdminI18n,
  useAdminWorkbench,
} from 'sdkwork-router-admin-core';
import type { AdminRouteKey } from 'sdkwork-router-admin-types';

import { MainLayout } from '../layouts/MainLayout';
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
const CommercialPage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-commercial')).CommercialPage,
}));
const PricingPage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-pricing')).PricingPage,
}));
const GatewayAccessPage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-apirouter')).GatewayAccessPage,
}));
const GatewayRateLimitsPage = lazy(async () => ({
  default: (await import('sdkwork-router-admin-apirouter')).GatewayRateLimitsPage,
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

function RouteStage({
  children,
  routeKey,
}: {
  children: ReactNode;
  routeKey: string;
}) {
  return (
    <div className="admin-shell-route-stage" data-route-key={routeKey} key={routeKey}>
      <div className="admin-shell-route-scroll">{children}</div>
    </div>
  );
}

function LoadingScreen() {
  const { t } = useAdminI18n();

  return (
    <div className="admin-shell-route-stage admin-shell-route-stage-loading">
      <div className="admin-shell-route-scroll">
        <div className="flex min-h-full flex-col items-center justify-center gap-4 text-center">
          <LoadingBlock label={t('Synchronizing operator workspace...')} />
          <p className="max-w-md text-sm text-[var(--sdk-color-text-secondary)]">
            {t('Restoring theme, session, and live control-plane state.')}
          </p>
        </div>
      </div>
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

function ProtectedRoute({ children }: { children: ReactNode }) {
  const location = useLocation();
  const { authResolved, sessionUser } = useAdminWorkbench();

  if (!authResolved) {
    return <LoadingScreen />;
  }

  if (!sessionUser) {
    return (
      <Navigate
        replace
        to={withRedirect(ROUTE_PATHS.LOGIN, `${location.pathname}${location.search}`)}
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
      <MainLayout>
        <RouteStage routeKey={routeKey}>
          <Suspense fallback={<LoadingScreen />}>{children}</Suspense>
        </RouteStage>
      </MainLayout>
    </ProtectedRoute>
  );
}

export function AppRoutes() {
  const location = useLocation();
  const navigate = useNavigate();
  const workbench = useAdminWorkbench();
  const {
    authResolved,
    handleCreateApiKey,
    handleCreateRoutingProfile,
    handleCreateRateLimitPolicy,
    handleDeleteApiKey,
    handleDeleteApiKeyGroup,
    handleDeleteChannel,
    handleDeleteChannelModel,
    handleDeleteCredential,
    handleDeleteModel,
    handleDeleteModelPrice,
    handleDeleteOperatorUser,
    handleDeletePortalUser,
    handleDeleteProject,
    handleDeleteProvider,
    handleDeleteTenant,
    handleLogin,
    handleReloadRuntimes,
    handleSaveChannel,
    handleSaveChannelModel,
    handleSaveCredential,
    handleSaveModel,
    handleSaveModelPrice,
    handleSaveApiKeyGroup,
    handleUpdateMarketingCampaignBudgetStatus,
    handleUpdateMarketingCampaignStatus,
    handleUpdateMarketingCouponCodeStatus,
    handleUpdateMarketingCouponTemplateStatus,
    handleSaveOperatorUser,
    handleSavePortalUser,
    handleSaveProject,
    handleSaveProvider,
    handleSaveTenant,
    handleToggleApiKeyGroup,
    handleToggleOperatorUser,
    handleTogglePortalUser,
    handleUpdateApiKey,
    handleUpdateApiKeyStatus,
    loading,
    refreshWorkspace,
    sessionUser,
    snapshot,
    status,
  } = workbench;

  const navigateToRoute = (routeKey: AdminRouteKey) => {
    navigate(adminRoutePathByKey[routeKey]);
  };

  const authRouteElement = !authResolved ? (
    <LoadingScreen />
  ) : (
    <AdminLoginPage
      isAuthenticated={Boolean(sessionUser)}
      loading={loading}
      onLogin={handleLogin}
      status={status}
    />
  );

  return (
    <Routes>
      <Route
        element={
          <Navigate
            replace
            to={withRedirect(ROUTE_PATHS.LOGIN, new URLSearchParams(location.search).get('redirect'))}
          />
        }
        path={ROUTE_PATHS.AUTH}
      />
      <Route element={authRouteElement} path={ROUTE_PATHS.LOGIN} />
      <Route element={authRouteElement} path={ROUTE_PATHS.REGISTER} />
      <Route element={authRouteElement} path={ROUTE_PATHS.FORGOT_PASSWORD} />
      <Route
        element={
          <ProtectedPage routeKey={location.pathname}>
            <OverviewPage onNavigate={navigateToRoute} snapshot={snapshot} />
          </ProtectedPage>
        }
        path={ROUTE_PATHS.OVERVIEW}
      />
      <Route
        element={
          <ProtectedPage routeKey={location.pathname}>
            <UsersPage
              onDeleteOperatorUser={handleDeleteOperatorUser}
              onDeletePortalUser={handleDeletePortalUser}
              onSaveOperatorUser={handleSaveOperatorUser}
              onSavePortalUser={handleSavePortalUser}
              onToggleOperatorUser={handleToggleOperatorUser}
              onTogglePortalUser={handleTogglePortalUser}
              snapshot={snapshot}
            />
          </ProtectedPage>
        }
        path={ROUTE_PATHS.USERS}
      />
      <Route
        element={
          <ProtectedPage routeKey={location.pathname}>
            <TenantsPage
              onCreateApiKey={handleCreateApiKey}
              onDeleteApiKey={handleDeleteApiKey}
              onDeleteProject={handleDeleteProject}
              onDeleteTenant={handleDeleteTenant}
              onSaveProject={handleSaveProject}
              onSaveTenant={handleSaveTenant}
              onUpdateApiKeyStatus={handleUpdateApiKeyStatus}
              snapshot={snapshot}
            />
          </ProtectedPage>
        }
        path={ROUTE_PATHS.TENANTS}
      />
      <Route
        element={
          <ProtectedPage routeKey={location.pathname}>
            <CouponsPage
              onUpdateMarketingCampaignBudgetStatus={handleUpdateMarketingCampaignBudgetStatus}
              onUpdateMarketingCampaignStatus={handleUpdateMarketingCampaignStatus}
              onUpdateMarketingCouponCodeStatus={handleUpdateMarketingCouponCodeStatus}
              onUpdateMarketingCouponTemplateStatus={handleUpdateMarketingCouponTemplateStatus}
              snapshot={snapshot}
            />
          </ProtectedPage>
        }
        path={ROUTE_PATHS.COUPONS}
      />
      <Route
        element={
          <ProtectedPage routeKey={location.pathname}>
            <CommercialPage snapshot={snapshot} />
          </ProtectedPage>
        }
        path={ROUTE_PATHS.COMMERCIAL}
      />
      <Route
        element={
          <ProtectedPage routeKey={location.pathname}>
            <PricingPage snapshot={snapshot} />
          </ProtectedPage>
        }
        path={ROUTE_PATHS.PRICING}
      />
      <Route
        element={<Navigate replace to={ROUTE_PATHS.API_ROUTER_API_KEYS} />}
        path={ROUTE_PATHS.API_ROUTER_ROOT}
      />
      <Route
        element={
          <ProtectedPage routeKey={location.pathname}>
            <GatewayAccessPage
              onCreateApiKey={handleCreateApiKey}
              onDeleteApiKey={handleDeleteApiKey}
              onDeleteApiKeyGroup={handleDeleteApiKeyGroup}
              onRefreshWorkspace={refreshWorkspace}
              onSaveApiKeyGroup={handleSaveApiKeyGroup}
              onToggleApiKeyGroup={handleToggleApiKeyGroup}
              onUpdateApiKey={handleUpdateApiKey}
              onUpdateApiKeyStatus={handleUpdateApiKeyStatus}
              snapshot={snapshot}
            />
          </ProtectedPage>
        }
        path={ROUTE_PATHS.API_ROUTER_API_KEYS}
      />
      <Route
        element={
          <ProtectedPage routeKey={location.pathname}>
            <GatewayRateLimitsPage
              onCreateRateLimitPolicy={handleCreateRateLimitPolicy}
              snapshot={snapshot}
            />
          </ProtectedPage>
        }
        path={ROUTE_PATHS.API_ROUTER_RATE_LIMITS}
      />
      <Route
        element={
          <ProtectedPage routeKey={location.pathname}>
            <GatewayRoutesPage
              onCreateRoutingProfile={handleCreateRoutingProfile}
              onDeleteProvider={handleDeleteProvider}
              onRefreshWorkspace={refreshWorkspace}
              onSaveProvider={handleSaveProvider}
              snapshot={snapshot}
            />
          </ProtectedPage>
        }
        path={ROUTE_PATHS.API_ROUTER_ROUTE_CONFIG}
      />
      <Route
        element={
          <ProtectedPage routeKey={location.pathname}>
            <GatewayModelMappingsPage snapshot={snapshot} />
          </ProtectedPage>
        }
        path={ROUTE_PATHS.API_ROUTER_MODEL_MAPPING}
      />
      <Route
        element={
          <ProtectedPage routeKey={location.pathname}>
            <GatewayUsagePage onRefreshWorkspace={refreshWorkspace} snapshot={snapshot} />
          </ProtectedPage>
        }
        path={ROUTE_PATHS.API_ROUTER_USAGE_RECORDS}
      />
      <Route
        element={
          <ProtectedPage routeKey={location.pathname}>
            <CatalogPage
              onDeleteChannel={handleDeleteChannel}
              onDeleteChannelModel={handleDeleteChannelModel}
              onDeleteCredential={handleDeleteCredential}
              onDeleteModel={handleDeleteModel}
              onDeleteModelPrice={handleDeleteModelPrice}
              onDeleteProvider={handleDeleteProvider}
              onSaveChannel={handleSaveChannel}
              onSaveChannelModel={handleSaveChannelModel}
              onSaveCredential={handleSaveCredential}
              onSaveModel={handleSaveModel}
              onSaveModelPrice={handleSaveModelPrice}
              onSaveProvider={handleSaveProvider}
              snapshot={snapshot}
            />
          </ProtectedPage>
        }
        path={ROUTE_PATHS.CATALOG}
      />
      <Route
        element={
          <ProtectedPage routeKey={location.pathname}>
            <TrafficPage snapshot={snapshot} />
          </ProtectedPage>
        }
        path={ROUTE_PATHS.TRAFFIC}
      />
      <Route
        element={
          <ProtectedPage routeKey={location.pathname}>
            <OperationsPage onReloadRuntimes={handleReloadRuntimes} snapshot={snapshot} />
          </ProtectedPage>
        }
        path={ROUTE_PATHS.OPERATIONS}
      />
      <Route
        element={
          <ProtectedPage routeKey={location.pathname}>
            <SettingsPage />
          </ProtectedPage>
        }
        path={ROUTE_PATHS.SETTINGS}
      />
      <Route
        element={<Navigate replace to={ROUTE_PATHS.OVERVIEW} />}
        path={ADMIN_ROUTE_PATHS.ROOT}
      />
      <Route
        element={<Navigate replace to={sessionUser ? ROUTE_PATHS.OVERVIEW : ROUTE_PATHS.LOGIN} />}
        path="*"
      />
    </Routes>
  );
}
