import type { ReactNode } from 'react';
import { Navigate, Route, Routes, useLocation, useNavigate } from 'react-router-dom';

import { AdminLoginPage } from 'sdkwork-router-admin-auth';
import {
  adminRoutePathByKey,
  useAdminWorkbench,
} from 'sdkwork-router-admin-core';
import { CatalogPage } from 'sdkwork-router-admin-catalog';
import { CouponsPage } from 'sdkwork-router-admin-coupons';
import { OperationsPage } from 'sdkwork-router-admin-operations';
import { OverviewPage } from 'sdkwork-router-admin-overview';
import { SettingsPage } from 'sdkwork-router-admin-settings';
import { TenantsPage } from 'sdkwork-router-admin-tenants';
import { TrafficPage } from 'sdkwork-router-admin-traffic';
import type { AdminRouteKey } from 'sdkwork-router-admin-types';
import { UsersPage } from 'sdkwork-router-admin-users';

import { ROUTE_PATHS } from './routePaths';

function PageFrame({
  children,
  routeKey,
}: {
  children: ReactNode;
  routeKey: string;
}) {
  return (
    <div key={routeKey} className="adminx-page-frame">
      {children}
    </div>
  );
}

function LoadingScreen() {
  return (
    <div className="adminx-shell-loading">
      <div className="adminx-shell-loading-orb" />
      <strong>Synchronizing operator workspace...</strong>
      <span>Restoring theme, session, and live control-plane state.</span>
    </div>
  );
}

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const location = useLocation();
  const { authResolved, sessionUser } = useAdminWorkbench();

  if (!authResolved) {
    return <LoadingScreen />;
  }

  if (!sessionUser) {
    return <Navigate to={ROUTE_PATHS.LOGIN} replace state={{ from: location }} />;
  }

  return <>{children}</>;
}

export function AppRoutes() {
  const location = useLocation();
  const navigate = useNavigate();
  const { authResolved, handleLogin, sessionUser, snapshot, status, loading } = useAdminWorkbench();
  const {
    handleCreateApiKey,
    handleDeleteApiKey,
    handleDeleteChannel,
    handleDeleteCoupon,
    handleDeleteCredential,
    handleDeleteModel,
    handleDeleteOperatorUser,
    handleDeletePortalUser,
    handleDeleteProject,
    handleDeleteProvider,
    handleDeleteTenant,
    handleReloadRuntimes,
    handleSaveChannel,
    handleSaveCoupon,
    handleSaveCredential,
    handleSaveModel,
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

  return (
    <Routes>
      <Route
        path={ROUTE_PATHS.LOGIN}
        element={
          !authResolved ? (
            <LoadingScreen />
          ) : sessionUser ? (
            <Navigate to={ROUTE_PATHS.OVERVIEW} replace />
          ) : (
            <PageFrame routeKey={location.pathname}>
              <AdminLoginPage status={status} loading={loading} onLogin={handleLogin} />
            </PageFrame>
          )
        }
      />
      <Route
        path={ROUTE_PATHS.OVERVIEW}
        element={
          <ProtectedRoute>
            <PageFrame routeKey={location.pathname}>
              <OverviewPage snapshot={snapshot} onNavigate={navigateToRoute} />
            </PageFrame>
          </ProtectedRoute>
        }
      />
      <Route
        path={ROUTE_PATHS.USERS}
        element={
          <ProtectedRoute>
            <PageFrame routeKey={location.pathname}>
              <UsersPage
                snapshot={snapshot}
                onSaveOperatorUser={handleSaveOperatorUser}
                onSavePortalUser={handleSavePortalUser}
                onToggleOperatorUser={handleToggleOperatorUser}
                onTogglePortalUser={handleTogglePortalUser}
                onDeleteOperatorUser={handleDeleteOperatorUser}
                onDeletePortalUser={handleDeletePortalUser}
              />
            </PageFrame>
          </ProtectedRoute>
        }
      />
      <Route
        path={ROUTE_PATHS.TENANTS}
        element={
          <ProtectedRoute>
            <PageFrame routeKey={location.pathname}>
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
            </PageFrame>
          </ProtectedRoute>
        }
      />
      <Route
        path={ROUTE_PATHS.COUPONS}
        element={
          <ProtectedRoute>
            <PageFrame routeKey={location.pathname}>
              <CouponsPage
                snapshot={snapshot}
                onSaveCoupon={handleSaveCoupon}
                onToggleCoupon={handleToggleCoupon}
                onDeleteCoupon={handleDeleteCoupon}
              />
            </PageFrame>
          </ProtectedRoute>
        }
      />
      <Route
        path={ROUTE_PATHS.CATALOG}
        element={
          <ProtectedRoute>
            <PageFrame routeKey={location.pathname}>
              <CatalogPage
                snapshot={snapshot}
                onSaveChannel={handleSaveChannel}
                onSaveProvider={handleSaveProvider}
                onSaveCredential={handleSaveCredential}
                onSaveModel={handleSaveModel}
                onDeleteChannel={handleDeleteChannel}
                onDeleteProvider={handleDeleteProvider}
                onDeleteCredential={handleDeleteCredential}
                onDeleteModel={handleDeleteModel}
              />
            </PageFrame>
          </ProtectedRoute>
        }
      />
      <Route
        path={ROUTE_PATHS.TRAFFIC}
        element={
          <ProtectedRoute>
            <PageFrame routeKey={location.pathname}>
              <TrafficPage snapshot={snapshot} />
            </PageFrame>
          </ProtectedRoute>
        }
      />
      <Route
        path={ROUTE_PATHS.OPERATIONS}
        element={
          <ProtectedRoute>
            <PageFrame routeKey={location.pathname}>
              <OperationsPage
                snapshot={snapshot}
                onReloadRuntimes={handleReloadRuntimes}
              />
            </PageFrame>
          </ProtectedRoute>
        }
      />
      <Route
        path={ROUTE_PATHS.SETTINGS}
        element={
          <ProtectedRoute>
            <PageFrame routeKey={location.pathname}>
              <SettingsPage />
            </PageFrame>
          </ProtectedRoute>
        }
      />
      <Route path="/" element={<Navigate to={ROUTE_PATHS.OVERVIEW} replace />} />
      <Route
        path="*"
        element={<Navigate to={sessionUser ? ROUTE_PATHS.OVERVIEW : ROUTE_PATHS.LOGIN} replace />}
      />
    </Routes>
  );
}
