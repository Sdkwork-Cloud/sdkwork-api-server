import { useLocation } from 'react-router-dom';

import { useAdminWorkbench } from 'sdkwork-router-admin-core';

import { AppHeader } from '../../components/AppHeader';
import { Sidebar } from '../../components/Sidebar';
import { AppRoutes } from '../router/AppRoutes';
import { ROUTE_PATHS } from '../router/routePaths';

export function MainLayout() {
  const location = useLocation();
  const { authResolved, sessionUser } = useAdminWorkbench();
  const isAuthRoute = location.pathname === ROUTE_PATHS.LOGIN;

  if (!authResolved && !sessionUser) {
    return (
      <div className="adminx-auth-stage">
        <AppRoutes />
      </div>
    );
  }

  if (isAuthRoute || !sessionUser) {
    return (
      <div className="adminx-auth-stage">
        <AppRoutes />
      </div>
    );
  }

  return (
    <div className="adminx-shell">
      <div className="adminx-shell-background" />
      <AppHeader />
      <div className="adminx-shell-body">
        <Sidebar />
        <main className="adminx-shell-main">
          <AppRoutes />
        </main>
      </div>
    </div>
  );
}
